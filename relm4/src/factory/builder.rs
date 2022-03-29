use super::{handle::FactoryHandle, DynamicIndex, FactoryComponent, FactoryView};

use crate::{shutdown, Receiver, Sender};

use std::cell::RefCell;
use std::rc::Rc;

use async_oneshot::oneshot;
use futures::FutureExt;

#[allow(missing_debug_implementations)]
pub(super) struct FactoryBuilder<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
    ParentMsg: 'static,
{
    pub(super) data: Rc<RefCell<C>>,
    pub(super) root_widget: C::Root,
    pub(super) input_tx: Sender<C::Input>,
    pub(super) input_rx: Receiver<C::Input>,
    pub(super) output_tx: Sender<C::Output>,
    pub(super) output_rx: Receiver<C::Output>,
}

impl<Widget, C, ParentMsg> FactoryBuilder<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
    ParentMsg: 'static,
{
    pub(super) fn new(index: &DynamicIndex, params: C::InitParams) -> Self {
        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_tx, output_rx) = crate::channel::<C::Output>();

        let data = Rc::new(RefCell::new(C::init_model(
            params, index, &input_tx, &output_tx,
        )));

        let root_widget = C::init_root();

        Self {
            data,
            root_widget,
            input_tx,
            input_rx,
            output_tx,
            output_rx,
        }
    }

    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub(super) fn launch<Transform>(
        self,
        index: &DynamicIndex,
        returned_widget: Widget::ReturnedWidget,
        parent_sender: &Sender<ParentMsg>,
        transform: Transform,
    ) -> FactoryHandle<Widget, C, ParentMsg>
    where
        Transform: Fn(C::Output) -> Option<ParentMsg> + 'static,
    {
        let Self {
            data,
            root_widget,
            input_tx,
            mut input_rx,
            output_tx,
            mut output_rx,
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

        // Sends messages from commands executed from the background.
        let (cmd_tx, mut cmd_rx) = crate::channel::<C::CommandOutput>();

        // Gets notifications when a component's model and view is updated externally.
        let (notifier, notifier_rx) = flume::bounded(0);

        let mut widgets = data.borrow_mut().init_widgets(
            index,
            &root_widget,
            &returned_widget,
            &input_tx,
            &output_tx,
        );

        // The source ID of the component's service will be sent through this once the root
        // widget has been iced, which will give the component one last chance to say goodbye.
        let (burn_notifier, burn_recipient) = oneshot::<gtk::glib::SourceId>();

        // Notifies the component's child commands that it is now deceased.
        let (death_notifier, death_recipient) = shutdown::channel();

        let input_tx_ = input_tx.clone();
        let runtime_data = data.clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        let id = crate::spawn_local(async move {
            let mut burn_notice = burn_recipient.fuse();
            loop {
                let notifier = notifier_rx.recv_async().fuse();
                let cmd = cmd_rx.recv().fuse();
                let input = input_rx.recv().fuse();

                futures::pin_mut!(cmd);
                futures::pin_mut!(input);
                futures::pin_mut!(notifier);

                let _ = futures::select!(
                    // Performs the model update, checking if the update requested a command.
                    // Runs that command asynchronously in the background using tokio.
                    message = input => {
                        if let Some(message) = message {
                            let mut model = runtime_data.borrow_mut();

                            if let Some(command) = model.update_with_view(&mut widgets, message, &input_tx_, &output_tx)
                            {
                                let recipient = death_recipient.clone();
                                crate::spawn(C::command(command, recipient, cmd_tx.clone()));
                            }
                        }
                    }

                    // Handles responses from a command.
                    message = cmd => {
                        if let Some(message) = message {
                            let mut model = runtime_data.borrow_mut();
                            model.update_cmd_with_view(&mut widgets, message, &input_tx_, &output_tx);
                        }
                    }

                    // Triggered when the model and view have been updated externally.
                    _ = notifier => {
                        let model = runtime_data.borrow_mut();
                        model.update_view(&mut widgets, &input_tx_, &output_tx);
                    }

                    // Triggered when the component is destroyed
                    id = burn_notice => {
                        let mut model = runtime_data.borrow_mut();

                        model.shutdown(&mut widgets, output_tx);

                        death_notifier.shutdown();

                        if let Ok(id) = id {
                            id.remove();
                        }

                        return
                    }
                );
            }
        });

        // Give back a type for controlling the component service.
        FactoryHandle {
            data,
            root_widget,
            returned_widget,
            input: input_tx,
            notifier: Sender(notifier),
            burner: crate::component::CompBurner {
                runtime_id: RefCell::new(Some(id)),
                burn_notifier,
            },
        }
    }
}
