// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::{Component, Fairing, Fuselage};
use gtk::prelude::*;
use std::marker::PhantomData;
use tokio::sync::mpsc;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct Bridge<Component, Root> {
    /// The root widget of the component.
    pub root: Root,

    pub(super) component: PhantomData<Component>,
}

impl<C: Component> Bridge<C, C::Root> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch(self, payload: C::Payload) -> Fairing<C::Root, C::Input, C::Output> {
        let Bridge { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, input_rx) = mpsc::unbounded_channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<C::Output>();

        // This channel enables forwards inputs to the internal service.
        let (inner_tx, mut inner_rx) = mpsc::unbounded_channel::<InnerMessage<C::Input>>();

        // Send a `Drop` message to the internal service.
        root.as_ref()
            .connect_destroy(gtk::glib::clone!(@strong inner_tx => move |_| {
                let _ = inner_tx.send(InnerMessage::Drop);
            }));

        // Constructs the initial model and view with the initial payload.
        let Fuselage {
            mut model,
            mut widgets,
        } = C::dock(payload, &root, &mut input_tx, &mut output_tx);

        // Forward `Self::Input` events to the internal service.
        let forwarder = crate::forward(input_rx, inner_tx, InnerMessage::Message);

        // The internal service which manages all input requests.
        let mut input_ = input_tx.clone();
        let service = async move {
            while let Some(event) = inner_rx.recv().await {
                match event {
                    InnerMessage::Message(event) => {
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        if let Some(command) = model.update(event, &mut input_, &mut output_tx) {
                            let input = input_.clone();
                            tokio::spawn(async move {
                                C::command(command, input).await;
                            });
                        }

                        model.update_view(&mut widgets, &mut input_, &mut output_tx);
                    }

                    InnerMessage::Drop => break,
                }
            }
        };

        // Start the internal service and its message forwarder.
        crate::spawn_local(async move {
            futures::future::join(forwarder, service).await;
        });

        Fairing {
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}

/// Used to drop the component's event loop when the managed widget is destroyed.
enum InnerMessage<T> {
    Drop,
    Message(T),
}
