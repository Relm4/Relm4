// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::super::*;
use super::*;
use futures::FutureExt;
use std::cell::RefCell;
use std::rc::Rc;

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

        // The main service receives `Self::Input` and `Self::CommandOutput` messages and applies
        // them to the model and view.
        let mut input_tx_ = input_tx.clone();
        let fuselage_ = watcher.clone();
        let id = crate::spawn_local(async move {
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
                            } = &mut *fuselage_.state.borrow_mut();

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
                            } = &mut *fuselage_.state.borrow_mut();

                            model.update_cmd(message, &mut input_tx_, &mut output_tx);
                            model.update_view(widgets, &mut input_tx_, &mut output_tx);
                        }
                    }

                    // Triggered when the model and view have been updated externally.
                    _ = notifier => {
                        let &mut ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *fuselage_.state.borrow_mut();

                        model.update_view(widgets, &mut input_tx_, &mut output_tx);
                    }
                );
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || id.remove());

        // Give back a type for controlling the component service.
        Connector {
            state: watcher,
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}
