use super::{FactoryComponent, FactoryHandle};

use crate::factory::{DataGuard, FactorySender, FactoryView};
use crate::shutdown::ShutdownSender;
use crate::{shutdown, GuardedReceiver, Receiver, Sender};

use std::any;

use tracing::info_span;

#[derive(Debug)]
pub(super) struct FactoryBuilder<C: FactoryComponent> {
    pub(super) data: Box<C>,
    pub(super) root_widget: C::Root,
    pub(super) component_sender: FactorySender<C>,
    pub(super) input_receiver: Receiver<C::Input>,
    pub(super) output_receiver: Receiver<C::Output>,
    pub(super) cmd_receiver: Receiver<C::CommandOutput>,
    pub(super) shutdown_notifier: ShutdownSender,
}

impl<C: FactoryComponent> FactoryBuilder<C> {
    pub(super) fn new(index: &C::Index, init: C::Init) -> Self {
        // Used for all events to be processed by this component's internal service.
        let (input_sender, input_receiver) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_sender, output_receiver) = crate::channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_sender, cmd_receiver) = crate::channel::<C::CommandOutput>();

        // Notifies the component's child commands that it is now deceased.
        let (shutdown_notifier, shutdown_receiver) = shutdown::channel();

        // Encapsulates the senders used by component methods.
        let component_sender =
            FactorySender::new(input_sender, output_sender, cmd_sender, shutdown_receiver);

        let data = Box::new(C::init_model(init, index, component_sender.clone()));
        let root_widget = data.init_root();

        Self {
            data,
            root_widget,
            component_sender,
            input_receiver,
            output_receiver,
            cmd_receiver,
            shutdown_notifier,
        }
    }

    /// Starts the component, passing ownership to a future attached to a [gtk::glib::MainContext].
    pub(super) fn launch<Transform>(
        self,
        index: &C::Index,
        returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
        parent_sender: &Sender<C::ParentInput>,
        transform: Transform,
    ) -> FactoryHandle<C>
    where
        Transform: Fn(C::Output) -> Option<C::ParentInput> + 'static,
    {
        let Self {
            mut data,
            root_widget,
            component_sender,
            input_receiver,
            output_receiver,
            cmd_receiver,
            shutdown_notifier,
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

        let widgets = Box::new(data.init_widgets(
            index,
            &root_widget,
            &returned_widget,
            component_sender.clone(),
        ));

        let input_sender = component_sender.input_sender().clone();
        let output_sender = component_sender.output_sender().clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        let data = DataGuard::new(
            data,
            widgets,
            shutdown_notifier,
            output_sender,
            |mut model, mut widgets| {
                async move {
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

                                model.update_with_view(&mut widgets, message, component_sender.clone());
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

                                model.update_cmd_with_view(&mut widgets, message, component_sender.clone());
                            }

                            // Triggered when the model and view have been updated externally.
                            _ = notifier => {
                                model.update_view(&mut widgets, component_sender.clone());
                            }
                        );
                    }
                }
            },
            C::shutdown,
        );

        // Give back a type for controlling the component service.
        FactoryHandle {
            data,
            root_widget,
            returned_widget,
            input: input_sender,
            notifier,
        }
    }
}
