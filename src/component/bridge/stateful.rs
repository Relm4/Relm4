// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::super::*;
use super::*;

impl<C: StatefulComponent> Bridge<C, C::Root> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch_stateful(self, payload: C::Payload) -> Fairing<C::Root, C::Input, C::Output> {
        let Bridge { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, mut input_rx) = mpsc::unbounded_channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<C::CommandOutput>();

        // Constructs the initial model and view with the initial payload.
        let Fuselage {
            mut model,
            mut widgets,
        } = C::dock(payload, &root, &mut input_tx, &mut output_tx);

        // The main service receives `Self::Input` and `Self::CommandOutput` messages and applies
        // them to the model and view.
        let mut input_tx_ = input_tx.clone();
        let id = crate::spawn_local(async move {
            loop {
                let input_future = input_rx.recv();
                let cmd_future = cmd_rx.recv();

                futures::pin_mut!(input_future);
                futures::pin_mut!(cmd_future);

                match futures::future::select(input_future, cmd_future).await {
                    // Performs the model update, checking if the update requested a command.
                    // Runs that command asynchronously in the background using tokio.
                    Either::Left((Some(message), _)) => {
                        if let Some(command) =
                            model.update(&mut widgets, message, &mut input_tx_, &mut output_tx)
                        {
                            let cmd_tx = cmd_tx.clone();
                            crate::spawn(async move {
                                if let Some(output) = C::command(command).await {
                                    let _ = cmd_tx.send(output);
                                }
                            });
                        }
                    }

                    // Responds to outputs received by commands.
                    Either::Right((Some(message), _)) => {
                        model.update_cmd(&mut widgets, message, &mut input_tx_, &mut output_tx);
                    }

                    _ => (),
                }
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || id.remove());

        // Give back a type for controlling the component service.
        Fairing {
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}
