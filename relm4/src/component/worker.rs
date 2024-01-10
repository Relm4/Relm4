// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use gtk::glib;
use tracing::info_span;

use crate::{
    Component, ComponentBuilder, ComponentParts, ComponentSender, GuardedReceiver, Receiver,
    RuntimeSenders, Sender, ShutdownOnDrop, SimpleComponent,
};
use std::fmt::Debug;
use std::{any, thread};

/// Receives inputs and outputs in the background.
///
/// All types that implement [`Worker`] will also implement
/// [`Component`] automatically.
///
/// If you need more flexibility when using workers, you can
/// simply implement [`Component`] instead and set the [`Component::Widgets`]
/// and [`Component::Root`] types both to `()`.
/// This will still allow you to use all worker related methods because internally
/// a worker is just seen as a [`Component`] without widgets.
pub trait Worker: Sized + Send + 'static {
    /// The initial parameters that will be used to build the worker state.
    type Init: 'static + Send;
    /// The type of inputs that this worker shall receive.
    type Input: 'static + Send + Debug;
    /// The type of outputs that this worker shall send.
    type Output: 'static + Send + Debug;

    /// Defines the initial state of the worker.
    fn init(init: Self::Init, sender: ComponentSender<Self>) -> Self;

    /// Defines how inputs will bep processed
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>);
}

impl<T> SimpleComponent for T
where
    T: Worker + 'static,
{
    type Root = ();
    type Widgets = ();

    type Init = <Self as Worker>::Init;
    type Input = <Self as Worker>::Input;
    type Output = <Self as Worker>::Output;

    fn init_root() -> Self::Root {}

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::init(init, sender);
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        Self::update(self, message, sender);
    }
}

impl<C> ComponentBuilder<C>
where
    C: Component<Root = (), Widgets = ()> + Send,
    C::Input: Send,
    C::Output: Send,
    C::CommandOutput: Send,
{
    /// Starts a worker on a separate thread,
    /// passing ownership to a future attached to a [gtk::glib::MainContext].
    pub fn detach_worker(self, payload: C::Init) -> WorkerHandle<C> {
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
        let component_sender = ComponentSender::new(
            input_sender.clone(),
            output_sender.clone(),
            cmd_sender,
            shutdown_recipient,
        );

        let mut state = C::init(payload, root, component_sender.clone());

        thread::spawn(move || {
            let context = glib::MainContext::thread_default().unwrap_or_default();

            // Spawns the component's service. It will receive both `Self::Input` and
            // `Self::CommandOutput` messages. It will spawn commands as requested by
            // updates, and send `Self::Output` messages externally.
            context.block_on(async move {
                let mut cmd = GuardedReceiver::new(cmd_receiver);
                let mut input = GuardedReceiver::new(input_receiver);

                loop {
                    futures::select!(
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        message = input => {
                            let ComponentParts {
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

                            model.update_with_view(widgets, message, component_sender.clone(), &root);
                        }

                        // Handles responses from a command.
                        message = cmd => {
                            let ComponentParts {
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

                            model.update_cmd_with_view(widgets, message, component_sender.clone(), &root);
                        },

                        // Triggered when the component is destroyed
                        _ = shutdown_event => {
                            let ComponentParts {
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
        WorkerHandle {
            sender: input_sender,
            receiver: output_receiver,
            shutdown_on_drop,
        }
    }
}

#[derive(Debug)]
/// Handle to a worker task in the background
pub struct WorkerHandle<W: Component> {
    // Sends inputs to the worker.
    sender: Sender<W::Input>,
    // Where the worker will send its outputs to.
    receiver: Receiver<W::Output>,
    // Shutdown the worker when this is dropped
    shutdown_on_drop: ShutdownOnDrop,
}

impl<W: Component> WorkerHandle<W>
where
    W::Input: 'static,
    W::Output: 'static,
{
    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<W::Input>, W::Output) + 'static>(
        self,
        mut func: F,
    ) -> WorkerController<W> {
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

        WorkerController {
            sender,
            shutdown_on_drop,
        }
    }

    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(W::Output) -> X) + 'static>(
        self,
        sender: &Sender<X>,
        transform: F,
    ) -> WorkerController<W> {
        let Self {
            sender: own_sender,
            receiver,
            shutdown_on_drop,
        } = self;

        crate::spawn_local(receiver.forward(sender.clone(), transform));
        WorkerController {
            sender: own_sender,
            shutdown_on_drop,
        }
    }

    /// Ignore outputs from the component and finish the builder.
    #[must_use]
    pub fn detach(self) -> WorkerController<W> {
        let Self {
            sender,
            shutdown_on_drop,
            ..
        } = self;

        WorkerController {
            sender,
            shutdown_on_drop,
        }
    }
}

/// Sends inputs to a worker. On drop, shuts down the worker.
#[derive(Debug)]
pub struct WorkerController<W: Component> {
    // Sends inputs to the worker.
    sender: Sender<W::Input>,
    // Shutdown the worker when this is dropped
    shutdown_on_drop: ShutdownOnDrop,
}

impl<W: Component> WorkerController<W> {
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
    /// In other words, dropping the [`WorkerController`] will not stop
    /// the runtime anymore, it will run until the app is closed.
    pub fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate();
    }
}
