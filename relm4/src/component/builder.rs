// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::message_broker::MessageBroker;
use super::{Component, ComponentParts, ComponentSenderInner, Connector, OnDestroy, StateWatcher};
use crate::{late_initialization, shutdown};
use crate::{Receiver, RelmContainerExt, RelmWidgetExt, Sender};
use async_oneshot::oneshot;
use futures::FutureExt;
use gtk::prelude::{GtkWindowExt, NativeDialogExt};
use std::any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info_span;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct ComponentBuilder<C: Component> {
    /// The root widget of the component.
    pub root: C::Root,

    pub(super) component: PhantomData<C>,
}

impl<C: Component> Default for ComponentBuilder<C> {
    /// Prepares a component for initialization.
    fn default() -> Self {
        ComponentBuilder {
            root: C::init_root(),
            component: PhantomData,
        }
    }
}

impl<C: Component> ComponentBuilder<C> {
    /// Configure the root widget before launching.
    #[must_use]
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
    #[must_use]
    pub fn attach_to(self, container: &impl RelmContainerExt) -> Self {
        container.container_add(self.root.as_ref());

        self
    }
}

impl<C: Component> ComponentBuilder<C>
where
    C::Root: AsRef<gtk::Window> + Clone,
{
    /// Set the component's root widget transient for a given window.
    /// This function doesn't require a [`gtk::Window`] as parameter,
    /// but instead uses [`RelmWidgetExt::toplevel_window()`] to retrieve the toplevel
    /// window of any [`gtk::Widget`].
    /// Therefore, you don't have to pass a window to every component.
    ///
    /// If the root widget is a native dialog, such as [`gtk::FileChooserNative`],
    /// you should use [`transient_for_native`][ComponentBuilder::transient_for_native] instead.
    #[must_use]
    pub fn transient_for(self, widget: impl AsRef<gtk::Widget>) -> Self {
        let widget = widget.as_ref().clone();
        let root = self.root.clone();
        late_initialization::register_callback(Box::new(move || {
            if let Some(window) = widget.toplevel_window() {
                root.as_ref().set_transient_for(Some(&window));
            } else {
                tracing::error!("Couldn't find root of transient widget")
            }
        }));

        self
    }
}

impl<C: Component> ComponentBuilder<C>
where
    C::Root: AsRef<gtk::NativeDialog> + Clone,
{
    /// Set the component's root widget transient for a given window.
    /// This function doesn't require a [`gtk::Window`] as parameter,
    /// but instead uses [`RelmWidgetExt::toplevel_window()`] to retrieve the toplevel
    /// window of any [`gtk::Widget`].
    /// Therefore, you don't have to pass a window to every component.
    ///
    /// Applicable to native dialogs only, such as [`gtk::FileChooserNative`].
    /// If the root widget is a non-native dialog, 
    /// you should use [`transient_for`][ComponentBuilder::transient_for] instead.
    #[must_use]
    pub fn transient_for_native(self, widget: impl AsRef<gtk::Widget>) -> Self {
        let widget = widget.as_ref().clone();
        let root = self.root.clone();
        late_initialization::register_callback(Box::new(move || {
            if let Some(window) = widget.toplevel_window() {
                root.as_ref().set_transient_for(Some(&window));
            } else {
                tracing::error!("Couldn't find root of transient widget")
            }
        }));

        self
    }
}

impl<C: Component> ComponentBuilder<C> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn launch(self, payload: C::Init) -> Connector<C> {
        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        self.launch_with_input_channel(payload, input_tx, input_rx)
    }

    /// Similar to [`launch()`](ComponentBuilder::launch) but also initializes a [`MessageBroker`].
    ///
    /// # Panics
    ///
    /// This method panics if the message broker was already initialized in another launch.
    pub fn launch_with_broker(self, payload: C::Init, broker: &MessageBroker<C>) -> Connector<C> {
        let (input_tx, input_rx) = broker.get_channel();
        self.launch_with_input_channel(
            payload,
            input_tx,
            input_rx.expect("Message broker launched multiple times"),
        )
    }

    fn launch_with_input_channel(
        self,
        payload: C::Init,
        input_tx: Sender<C::Input>,
        input_rx: Receiver<C::Input>,
    ) -> Connector<C> {
        let ComponentBuilder { root, .. } = self;

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
            state: RefCell::new(C::init(payload, &root, component_sender.clone())),
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

                futures::select!(
                    // Performs the model update, checking if the update requested a command.
                    // Runs that command asynchronously in the background using tokio.
                    message = input => {
                        if let Some(message) = message {
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            let span = info_span!(
                                "update_with_view",
                                input=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_with_view(widgets, message, component_sender.clone());
                        }
                    }

                    // Handles responses from a command.
                    message = cmd => {
                        if let Some(message) = message {
                            let &mut ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut *watcher_.state.borrow_mut();

                            let span = info_span!(
                                "update_cmd_with_view",
                                cmd_output=?message,
                                component=any::type_name::<C>(),
                                id=model.id(),
                            );
                            let _enter = span.enter();

                            model.update_cmd_with_view(widgets, message, component_sender.clone());
                        }
                    }

                    // Triggered when the model and view have been updated externally.
                    _ = notifier => {
                        let &mut ComponentParts {
                            ref mut model,
                            ref mut widgets,
                        } = &mut *watcher_.state.borrow_mut();

                        model.update_view(widgets, component_sender.clone());
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
