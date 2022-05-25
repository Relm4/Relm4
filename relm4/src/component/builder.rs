// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::{Component, ComponentParts, ComponentSenderInner, Connector, OnDestroy, StateWatcher};
use crate::shutdown;
use crate::RelmContainerExt;
use async_oneshot::oneshot;
use futures::FutureExt;
use gtk::prelude::GtkWindowExt;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct ComponentBuilder<C: Component> {
    /// The root widget of the component.
    pub root: C::Root,

    pub(super) component: PhantomData<C>,
}

impl<C: Component> ComponentBuilder<C> {
    #[allow(clippy::new_without_default)]
    /// Prepares a component for initialization.
    pub fn new() -> Self {
        ComponentBuilder {
            root: C::init_root(),
            component: PhantomData,
        }
    }

    /// Configure the root widget before launching.
    pub fn update_root<F: FnOnce(&mut C::Root)>(mut self, func: F) -> Self {
        func(&mut self.root);
        self
    }

    /// Access the root widget before the component is initialized.
    pub fn widget(&self) -> &C::Root {
        &self.root
    }
}

impl<C: Component> ComponentBuilder<C>
where
    C::Root: AsRef<gtk::Widget>,
{
    /// Attach the component's root widget to a given container.
    pub fn attach_to(self, container: &impl RelmContainerExt) -> Self {
        container.container_add(self.root.as_ref());

        self
    }
}

impl<C: Component> ComponentBuilder<C>
where
    C::Root: AsRef<gtk::Window>,
{
    /// Set the component's root widget transient for a given window.
    pub fn transient_for(self, window: impl AsRef<gtk::Window>) -> Self {
        self.root.as_ref().set_transient_for(Some(window.as_ref()));

        self
    }
}

impl<C: Component> ComponentBuilder<C> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch(self, payload: C::InitParams) -> Connector<C> {
        let ComponentBuilder { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_tx, output_rx) = crate::channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, cmd_rx) = crate::channel::<C::CommandOutput>();

        // Gets notifications when a component's model and view is updated externally.
        let (notifier, notifier_rx) = flume::bounded(0);

        // Notifies the component's child commands that it is now deceased.
        let (death_notifier, death_recipient) = shutdown::channel();

        // Encapsulates the senders used by component methods.
        let component_sender = Arc::new(ComponentSenderInner {
            command: cmd_tx,
            input: input_tx.clone(),
            output: output_tx.clone(),
            shutdown: death_recipient,
        });

        // Constructs the initial model and view with the initial payload.
        let watcher = Rc::new(StateWatcher {
            state: RefCell::new(C::init(payload, &root, &component_sender)),
            notifier,
        });

        // The source ID of the component's service will be sent through this once the root
        // widget has been iced, which will give the component one last chance to say goodbye.
        let (mut burn_notifier, burn_recipient) = oneshot::<gtk::glib::SourceId>();

        let watcher_ = watcher.clone();

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
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            model.update_with_view(widgets, message, &component_sender);
                        }
                    }

                    // Handles responses from a command.
                    message = cmd => {
                        if let Some(message) = message {
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            model.update_cmd_with_view(widgets, message, &component_sender);
                        }
                    }

                    // Triggered when the model and view have been updated externally.
                    _ = notifier => {
                        let &mut ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *watcher_.state.borrow_mut();

                        model.update_view(widgets, &component_sender);
                    }

                    // Triggered when the component is destroyed
                    id = burn_notice => {
                        let ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *watcher_.state.borrow_mut();

                        model.shutdown(widgets, output_tx);

                        death_notifier.shutdown();

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
