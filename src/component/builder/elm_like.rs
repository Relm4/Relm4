// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::super::*;
use super::*;
use futures::FutureExt;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::oneshot;

impl<C: Component> ComponentBuilder<C, C::Root> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch(
        self,
        payload: C::InitParams,
    ) -> Connector<C, C::Root, C::Widgets, C::Input, C::Output> {
        let ComponentBuilder { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, mut input_rx) = mpsc::unbounded_channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<C::CommandOutput>();

        // Gets notifications when a component's model and view is updated externally.
        let notifier = Rc::new(tokio::sync::Notify::new());

        // Constructs the initial model and view with the initial payload.
        let watcher = Rc::new(StateWatcher {
            state: RefCell::new(C::init_parts(payload, &root, &mut input_tx, &mut output_tx)),
            notifier: notifier.clone(),
        });

        // The source ID of the component's service will be sent through this once the root
        // widget has been iced, which will give the component one last chance to say goodbye.
        let (burn_notifier, burn_recipient) = oneshot::channel::<gtk::glib::SourceId>();

        let mut input_tx_ = input_tx.clone();
        let watcher_ = watcher.clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        let id = crate::spawn_local(async move {
            let mut burn_notice = burn_recipient.fuse();
            loop {
                let notifier = notifier.notified().fuse();
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
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            if let Some(command) = model.update(message, &mut input_tx_, &mut output_tx)
                            {
                                let input = cmd_tx.clone();
                                crate::spawn(async move {
                                    if let Some(message) = C::command(command).await {
                                        let _ = input.send(message);
                                    }
                                });
                            }

                            model.update_view(widgets, &mut input_tx_, &mut output_tx);
                        }
                    }

                    // Handles responses from a command.
                    message = cmd => {
                        if let Some(message) = message {
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            model.update_cmd(message, &mut input_tx_, &mut output_tx);
                            model.update_view(widgets, &mut input_tx_, &mut output_tx);
                        }
                    }

                    // Triggered when the model and view have been updated externally.
                    _ = notifier => {
                        let &mut ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *watcher_.state.borrow_mut();

                        model.update_view(widgets, &mut input_tx_, &mut output_tx);
                    }

                    // Triggered when the component is destroyed
                    id = burn_notice => {
                        let ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *watcher_.state.borrow_mut();

                        model.shutdown(widgets, output_tx);

                        if let Ok(id) = id {
                            id.remove();
                        }

                        return
                    }
                );
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || {
            let _ = burn_notifier.send(id);
        });

        // Give back a type for controlling the component service.
        Connector {
            state: watcher,
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}
