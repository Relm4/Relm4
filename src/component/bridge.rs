// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::{Component, Fairing, Fuselage, OnDestroy, StatefulComponent};
use crate::RelmContainerExt;
use std::marker::PhantomData;
use tokio::sync::mpsc;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct Bridge<Component, Root> {
    /// The root widget of the component.
    pub root: Root,

    pub(super) component: PhantomData<Component>,
}

impl<Component, Root: AsRef<gtk::Widget>> Bridge<Component, Root> {
    /// Attach the component's root widget to a given container.
    pub fn attach_to(self, container: &impl RelmContainerExt) -> Self {
        container.container_add(self.root.as_ref());

        self
    }
}

impl<C: Component> Bridge<C, C::Root> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch(self, payload: C::Payload) -> Fairing<C::Root, C::Input, C::Output> {
        let Bridge { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, mut input_rx) = mpsc::unbounded_channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<C::Output>();

        // Constructs the initial model and view with the initial payload.
        let Fuselage {
            mut model,
            mut widgets,
        } = C::dock(payload, &root, &mut input_tx, &mut output_tx);

        // The internal service which manages all input requests.
        let mut input_ = input_tx.clone();

        // Start the internal service.
        let id = crate::spawn_local(async move {
            while let Some(event) = input_rx.recv().await {
                // Performs the model update, checking if the update requested a command.
                // Runs that command asynchronously in the background using tokio.
                if let Some(command) = model.update(event, &mut input_, &mut output_tx) {
                    let input = input_.clone();
                    crate::spawn(async move {
                        C::command(command, input).await;
                    });
                }

                model.update_view(&mut widgets, &mut input_, &mut output_tx);
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || id.remove());

        Fairing {
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}

impl<C: StatefulComponent> Bridge<C, C::Root> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch_stateful(self, payload: C::Payload) -> Fairing<C::Root, C::Input, C::Output> {
        let Bridge { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, mut input_rx) = mpsc::unbounded_channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<C::Output>();

        // Constructs the initial model and view with the initial payload.
        let Fuselage {
            mut model,
            mut widgets,
        } = C::dock(payload, &root, &mut input_tx, &mut output_tx);

        // The internal service which manages all input requests.
        let mut input_ = input_tx.clone();
        let id = crate::spawn_local(async move {
            while let Some(event) = input_rx.recv().await {
                // Performs the model update, checking if the update requested a command.
                // Runs that command asynchronously in the background using tokio.
                if let Some(command) =
                    model.update(&mut widgets, event, &mut input_, &mut output_tx)
                {
                    let input = input_.clone();
                    crate::spawn(async move {
                        C::command(command, input).await;
                    });
                }
            }
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || id.remove());

        Fairing {
            widget: root,
            sender: input_tx,
            receiver: output_rx,
        }
    }
}
