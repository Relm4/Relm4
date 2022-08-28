use super::data_guard::DataGuard;
use super::FactoryComponentSender;
use super::{handle::FactoryHandle, DynamicIndex, FactoryComponent, FactoryView};

use crate::component::ComponentSenderInner;
use crate::shutdown::ShutdownSender;
use crate::{shutdown, OnDestroy, Receiver, Sender};

use std::any;
use std::sync::Arc;

use async_oneshot::oneshot;
use futures::FutureExt;
use tracing::info_span;

#[derive(Debug)]
pub(super) struct FactoryBuilder<C: FactoryComponent> {
    pub(super) data: Box<C>,
    pub(super) root_widget: C::Root,
    pub(super) component_sender: FactoryComponentSender<C>,
    pub(super) input_rx: Receiver<C::Input>,
    pub(super) output_rx: Receiver<C::Output>,
    pub(super) cmd_rx: Receiver<C::CommandOutput>,
    pub(super) death_notifier: ShutdownSender,
}

impl<C: FactoryComponent> FactoryBuilder<C> {
    pub(super) fn new(index: &DynamicIndex, params: C::Init) -> Self {
        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_tx, output_rx) = crate::channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, cmd_rx) = crate::channel::<C::CommandOutput>();

        // Notifies the component's child commands that it is now deceased.
        let (death_notifier, death_recipient) = shutdown::channel();

        // Encapsulates the senders used by component methods.
        let component_sender = Arc::new(ComponentSenderInner {
            command: cmd_tx,
            input: input_tx,
            output: output_tx,
            shutdown: death_recipient,
        });

        let data = Box::new(C::init_model(params, index, component_sender.clone()));
        let root_widget = data.init_root();

        Self {
            data,
            root_widget,
            component_sender,
            input_rx,
            output_rx,
            cmd_rx,
            death_notifier,
        }
    }

    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub(super) fn launch<Transform>(
        self,
        index: &DynamicIndex,
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
            input_rx,
            output_rx,
            cmd_rx,
            death_notifier,
        } = self;

        let forward_sender = parent_sender.0.clone();
        crate::spawn_local(async move {
            while let Some(msg) = output_rx.recv().await {
                if let Some(new_msg) = transform(msg) {
                    if forward_sender.send(new_msg).is_err() {
                        break;
                    }
                }
            }
        });

        // Gets notifications when a component's model and view is updated externally.
        let (notifier, notifier_rx) = flume::bounded(0);

        // The source ID of the component's service will be sent through this once the root
        // widget has been iced, which will give the component one last chance to say goodbye.
        let (mut burn_notifier, burn_recipient) = oneshot::<gtk::glib::SourceId>();

        let mut widgets = data.init_widgets(
            index,
            &root_widget,
            &returned_widget,
            component_sender.clone(),
        );

        let input_tx = component_sender.input.clone();
        let output_tx = component_sender.output.clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        let (data, on_destroy_id) = DataGuard::new(data, |mut model| {
            async move {
                let mut burn_notice = burn_recipient.fuse();
                loop {
                    let notifier = notifier_rx.recv_async().fuse();
                    let cmd = cmd_rx.recv().fuse();
                    let input = input_rx.recv().fuse();

                    futures::pin_mut!(cmd);
                    futures::pin_mut!(input);
                    futures::pin_mut!(notifier);

                    futures::select!(
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        message = input => {
                            if let Some(message) = message {
                                let span = info_span!(
                                    "update_with_view",
                                    input=?message,
                                    component=any::type_name::<C>(),
                                    id=model.id(),
                                );
                                let _enter = span.enter();

                                model.update_with_view(&mut widgets, message, component_sender.clone());
                            }
                        }

                        // Handles responses from a command.
                        message = cmd => {
                            if let Some(message) = message {
                                let span = info_span!(
                                    "update_cmd_with_view",
                                    cmd_output=?message,
                                    component=any::type_name::<C>(),
                                    id=model.id(),
                                );
                                let _enter = span.enter();

                                model.update_cmd_with_view(&mut widgets, message, component_sender.clone());
                            }
                        }

                        // Triggered when the model and view have been updated externally.
                        _ = notifier => {
                            model.update_view(&mut widgets, component_sender.clone());
                        }

                        // Triggered when the component is destroyed
                        id = burn_notice => {
                            model.shutdown(&mut widgets, output_tx);

                            death_notifier.shutdown();

                            if let Ok(id) = id {
                                id.remove();
                            }

                            return
                        }
                    );
                }
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        let root_widget_ = root_widget.clone();
        root_widget_.on_destroy(move || {
            if let Some(id) = on_destroy_id.take() {
                let _ = burn_notifier.send(id);
            }
        });

        // Give back a type for controlling the component service.
        FactoryHandle {
            data,
            root_widget,
            returned_widget,
            input: input_tx,
            notifier: Sender(notifier),
        }
    }
}
