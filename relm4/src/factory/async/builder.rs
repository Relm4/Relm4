use std::any;

use tracing::info_span;

use super::future_data::AsyncData;
use super::{AsyncFactoryComponent, AsyncFactoryHandle};

use crate::factory::{DataGuard, DynamicIndex, FactoryView};
use crate::runtime_util::GuardedReceiver;
use crate::sender::AsyncFactoryComponentSender;
use crate::shutdown::ShutdownSender;
use crate::{shutdown, Receiver, Sender};

pub(super) struct AsyncFactoryBuilder<C: AsyncFactoryComponent> {
    init: C::Init,
    pub(super) root_widget: C::Root,
    pub(super) component_sender: AsyncFactoryComponentSender<C>,
    input_receiver: Receiver<C::Input>,
    output_receiver: Receiver<C::Output>,
    cmd_receiver: Receiver<C::CommandOutput>,
    shutdown_notifier: ShutdownSender,
}

impl<C: AsyncFactoryComponent> AsyncFactoryBuilder<C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    pub(super) fn new(init: C::Init) -> Self {
        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_tx, output_rx) = crate::channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, cmd_rx) = crate::channel::<C::CommandOutput>();

        // Notifies the component's child commands that it is now deceased.
        let (shutdown_notifier, shutdown_receiver) = shutdown::channel();

        // Encapsulates the senders used by component methods.
        let component_sender =
            AsyncFactoryComponentSender::new(input_tx, output_tx, cmd_tx, shutdown_receiver);

        let root_widget = C::init_root();

        Self {
            init,
            root_widget,
            component_sender,
            input_receiver: input_rx,
            output_receiver: output_rx,
            cmd_receiver: cmd_rx,
            shutdown_notifier,
        }
    }

    /// Starts the component, passing ownership to a future attached to a [gtk::glib::MainContext].
    pub(super) fn launch<Transform>(
        self,
        index: &DynamicIndex,
        returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
        parent_sender: &Sender<C::ParentInput>,
        transform: Transform,
    ) -> AsyncFactoryHandle<C>
    where
        Transform: Fn(C::Output) -> Option<C::ParentInput> + 'static,
    {
        let Self {
            mut root_widget,
            component_sender,
            input_receiver,
            output_receiver,
            cmd_receiver,
            shutdown_notifier,
            init,
        } = self;

        let forward_sender = parent_sender.0.clone();
        crate::spawn_local(async move {
            while let Some(msg) = output_receiver.recv().await {
                if let Some(new_msg) = transform(msg) {
                    if forward_sender.send(new_msg).is_err() {
                        break;
                    }
                }
            }
        });

        // Gets notifications when a component's model and view is updated externally.
        let (notifier, notifier_receiver) = crate::channel();

        let input_tx = component_sender.input_sender().clone();

        let future_receiver = {
            let index = index.clone();
            let (future_sender, future_receiver) = crate::channel();

            let future_data = FutureData {
                shutdown_notifier,
                index: index.clone(),
                component_sender: component_sender.clone(),
                root: root_widget.clone(),
                returned_widget: returned_widget.clone(),
                input_receiver,
                cmd_receiver,
                notifier_receiver,
            };

            crate::spawn_local(async move {
                let data = C::init_model(init, &index, component_sender).await;
                let data_guard = future_data.start_runtime(data);
                future_sender.send(data_guard);
            });
            future_receiver
        };

        C::temporary_init(&mut root_widget);

        let data = AsyncData::new(future_receiver);

        // Give back a type for controlling the component service.
        AsyncFactoryHandle {
            data,
            root_widget,
            returned_widget,
            input: input_tx,
            notifier,
        }
    }
}

impl<C: AsyncFactoryComponent> std::fmt::Debug for AsyncFactoryBuilder<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncFactoryBuilder")
            .field("init", &"<C::Init>")
            .field("root_widget", &self.root_widget)
            .field("component_sender", &"<AsyncComponentSender<C>>")
            .field("input_receiver", &self.input_receiver)
            .field("output_receiver", &self.output_receiver)
            .field("cmd_receiver", &self.cmd_receiver)
            .field("shutdown_notifier", &self.shutdown_notifier)
            .finish()
    }
}

struct FutureData<C: AsyncFactoryComponent> {
    shutdown_notifier: ShutdownSender,
    index: DynamicIndex,
    component_sender: AsyncFactoryComponentSender<C>,
    root: C::Root,
    returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
    input_receiver: Receiver<C::Input>,
    cmd_receiver: Receiver<C::CommandOutput>,
    notifier_receiver: Receiver<()>,
}

impl<C: AsyncFactoryComponent> FutureData<C> {
    fn start_runtime(self, data: C) -> DataGuard<C, C::Widgets, C::Output> {
        let Self {
            shutdown_notifier,
            index,
            component_sender,
            root,
            returned_widget,
            cmd_receiver,
            input_receiver,
            notifier_receiver,
        } = self;

        let mut data = Box::new(data);
        let widgets =
            Box::new(data.init_widgets(&index, &root, &returned_widget, component_sender.clone()));

        let output_sender = component_sender.output_sender().clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        DataGuard::new(
            data,
            widgets,
            shutdown_notifier,
            output_sender,
            |mut model, mut widgets| async move {
                let mut notifier = GuardedReceiver::new(notifier_receiver);
                let mut cmd = GuardedReceiver::new(cmd_receiver);
                let mut input = GuardedReceiver::new(input_receiver);
                loop {
                    futures::select!(
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        message = input => {
                            let span = info_span!(
                                "update_with_view",
                                input=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_with_view(&mut widgets, message, component_sender.clone()).await;
                        }

                        // Handles responses from a command.
                        message = cmd => {
                            let span = info_span!(
                                "update_cmd_with_view",
                                cmd_output=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_cmd_with_view(&mut widgets, message, component_sender.clone()).await;
                        }

                        // Triggered when the model and view have been updated externally.
                        _ = notifier => {
                            model.update_view(&mut widgets, component_sender.clone());
                        }
                    );
                }
            },
            C::shutdown,
        )
    }
}
