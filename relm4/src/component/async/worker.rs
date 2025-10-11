use gtk::glib;
use tracing::info_span;

use crate::{
    AsyncComponentSender, GuardedReceiver, Receiver, RuntimeSenders, Sender, ShutdownOnDrop,
    component::AsyncComponentBuilder,
    prelude::{AsyncComponent, AsyncComponentParts, SimpleAsyncComponent},
};
use std::{any, fmt::Debug, thread};

/// Asynchronous variant of [`Worker`].
///
/// All types that implement [`AsyncWorker`] will also implement
/// [`AsyncComponent`] automatically with `Root = ()` and `Widgets = ()`.
///
/// This is the async equivalent of [`Worker`], providing the same
/// background processing capabilities but with async/await support.
pub trait AsyncWorker: Sized + Send + 'static {
    /// The initial parameters that will be used to build the worker state.
    type Init: 'static + Send;
    /// The type of inputs that this worker shall receive.
    type Input: 'static + Send + Debug;
    /// The type of outputs that this worker shall send.
    type Output: 'static + Send + Debug;

    /// Defines the initial state of the worker.
    fn init(
        init: Self::Init,
        sender: AsyncComponentSender<Self>,
    ) -> impl Future<Output = Self> + Send;

    /// Defines how inputs will be processed asynchronously.
    fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
    ) -> impl Future<Output = ()> + Send;
}

impl<T> SimpleAsyncComponent for T
where
    T: AsyncWorker + 'static,
{
    type Root = ();
    type Widgets = ();

    type Init = <Self as AsyncWorker>::Init;
    type Input = <Self as AsyncWorker>::Input;
    type Output = <Self as AsyncWorker>::Output;

    fn init_root() -> Self::Root {}

    async fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self::init(init, sender).await;
        AsyncComponentParts { model, widgets: () }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        Self::update(self, message, sender).await;
    }
}

impl<C> AsyncComponentBuilder<C>
where
    C: AsyncComponent<Root = (), Widgets = ()> + Send,
    C::Input: Send,
    C::Output: Send,
    C::CommandOutput: Send,
{
    /// Starts an async worker on a separate thread,
    /// passing ownership to a future attached to a [gtk::glib::MainContext].
    pub fn detach_async_worker(self, payload: C::Init) -> AsyncWorkerHandle<C>
    where
        <C as AsyncComponent>::Init: Send,
    {
        let Self { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (input_sender, input_receiver) = crate::channel::<C::Input>();

        let RuntimeSenders {
            output_sender,
            output_receiver,
            cmd_sender,
            cmd_receiver,
            shutdown_notifier,
            shutdown_recipient,
            shutdown_on_drop,
            mut shutdown_event,
        } = RuntimeSenders::<C::Output, C::CommandOutput>::new();

        // Encapsulates the senders used by component methods.
        let component_sender = AsyncComponentSender::new(
            input_sender.clone(),
            output_sender.clone(),
            cmd_sender,
            shutdown_recipient,
        );

        thread::spawn(move || {
            let context = glib::MainContext::thread_default().unwrap_or_default();

            // Spawns the component's service. It will receive both `Self::Input` and
            // `Self::CommandOutput` messages. It will spawn commands as requested by
            // updates, and send `Self::Output` messages externally.
            context.block_on(async move {
                let mut state = C::init(payload, root, component_sender.clone()).await;
                let mut cmd = GuardedReceiver::new(cmd_receiver);
                let mut input = GuardedReceiver::new(input_receiver);

                loop {
                    futures::select!(
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        message = input => {
                            let AsyncComponentParts {
                                model,
                                widgets,
                            } = &mut state;

                            let span = info_span!(
                                "update_with_view",
                                input=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_with_view(widgets, message, component_sender.clone(), &root).await;
                        }

                        // Handles responses from a command.
                        message = cmd => {
                            let AsyncComponentParts {
                                model,
                                widgets,
                            } = &mut state;

                            let span = info_span!(
                                "update_cmd_with_view",
                                cmd_output=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_cmd_with_view(widgets, message, component_sender.clone(), &root).await;
                        },

                        // Triggered when the component is destroyed
                        _ = shutdown_event => {
                            let AsyncComponentParts {
                                model,
                                widgets,
                            } = &mut state;

                            model.shutdown(widgets, output_sender);

                            shutdown_notifier.shutdown();

                            return;
                        }
                    );
                }
            });
        });

        // Give back a type for controlling the component service.
        AsyncWorkerHandle {
            sender: input_sender,
            receiver: output_receiver,
            shutdown_on_drop,
        }
    }
}

/// Handle to an async worker task in the background
#[derive(Debug)]
pub struct AsyncWorkerHandle<W: AsyncComponent> {
    /// Sends inputs to the worker.
    sender: Sender<W::Input>,
    /// Where the worker will send its outputs to.
    receiver: Receiver<W::Output>,
    /// Shutdown the worker when this is dropped.
    shutdown_on_drop: ShutdownOnDrop,
}

impl<W: AsyncComponent> AsyncWorkerHandle<W>
where
    W::Input: 'static,
    W::Output: 'static,
{
    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<W::Input>, W::Output) + 'static>(
        self,
        mut func: F,
    ) -> AsyncWorkerController<W> {
        let Self {
            sender,
            receiver,
            shutdown_on_drop,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        AsyncWorkerController {
            sender,
            shutdown_on_drop,
        }
    }

    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(W::Output) -> X) + 'static>(
        self,
        sender: &Sender<X>,
        transform: F,
    ) -> AsyncWorkerController<W> {
        let Self {
            sender: own_sender,
            receiver,
            shutdown_on_drop,
        } = self;

        crate::spawn_local(receiver.forward(sender.clone(), transform));
        AsyncWorkerController {
            sender: own_sender,
            shutdown_on_drop,
        }
    }

    /// Ignore outputs from the component and finish the builder.
    #[must_use]
    pub fn detach(self) -> AsyncWorkerController<W> {
        let Self {
            sender,
            shutdown_on_drop,
            ..
        } = self;

        AsyncWorkerController {
            sender,
            shutdown_on_drop,
        }
    }
}

/// Sends inputs to an async worker. On drop, shuts down the worker.
#[derive(Debug)]
pub struct AsyncWorkerController<W: AsyncComponent> {
    /// Sends inputs to the worker.
    sender: Sender<W::Input>,
    /// Shutdown the worker when this is dropped.
    shutdown_on_drop: ShutdownOnDrop,
}

impl<W: AsyncComponent> AsyncWorkerController<W> {
    /// Emits an input to the component.
    pub fn emit(&self, event: W::Input) {
        self.sender.send(event).unwrap();
    }

    /// Provides access to the component's sender.
    #[must_use]
    pub const fn sender(&self) -> &Sender<W::Input> {
        &self.sender
    }

    /// Dropping this type will usually stop the runtime of the worker.
    /// With this method you can give the runtime a static lifetime.
    /// In other words, dropping the [`AsyncWorkerController`] will not stop
    /// the runtime anymore, it will run until the app is closed.
    pub fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate();
    }
}
