// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::super::MessageBroker;
use super::{AsyncComponent, AsyncComponentParts, AsyncConnector};
use crate::channel::AsyncComponentSender;
use crate::{
    late_initialization, GuardedReceiver, Receiver, RelmContainerExt, RelmWidgetExt,
    RuntimeSenders, Sender,
};
use gtk::glib;
use gtk::prelude::{GtkWindowExt, NativeDialogExt};
use std::any;
use std::marker::PhantomData;
use tokio::sync::oneshot;
use tracing::info_span;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct AsyncComponentBuilder<C: AsyncComponent> {
    /// The root widget of the component.
    pub root: C::Root,
    priority: glib::Priority,

    pub(super) component: PhantomData<C>,
}

impl<C: AsyncComponent> Default for AsyncComponentBuilder<C> {
    /// Prepares a component for initialization.
    fn default() -> Self {
        Self {
            root: C::init_root(),
            priority: glib::Priority::default(),
            component: PhantomData,
        }
    }
}

impl<C: AsyncComponent> AsyncComponentBuilder<C> {
    /// Configure the root widget before launching.
    #[must_use]
    pub fn update_root<F: FnOnce(&mut C::Root)>(mut self, func: F) -> Self {
        func(&mut self.root);
        self
    }

    /// Access the root widget before the component is initialized.
    pub const fn widget(&self) -> &C::Root {
        &self.root
    }

    /// Change the priority at which the messages of this component
    /// are handled.
    ///
    /// + Use [`glib::PRIORITY_HIGH`] for high priority event sources.
    /// + Use [`glib::PRIORITY_LOW`] for very low priority background tasks.
    /// + Use [`glib::PRIORITY_DEFAULT_IDLE`] for default priority idle functions.
    /// + Use [`glib::PRIORITY_HIGH_IDLE`] for high priority idle functions.
    pub fn priority(mut self, priority: glib::Priority) -> Self {
        self.priority = priority;
        self
    }
}

impl<C: AsyncComponent> AsyncComponentBuilder<C>
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

impl<C: AsyncComponent> AsyncComponentBuilder<C>
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
    /// you should use [`transient_for_native`][AsyncComponentBuilder::transient_for_native] instead.
    #[must_use]
    pub fn transient_for(self, widget: impl AsRef<gtk::Widget>) -> Self {
        let widget = widget.as_ref().clone();
        let root = self.root.clone();
        late_initialization::register_callback(Box::new(move || {
            if let Some(window) = widget.toplevel_window() {
                root.as_ref().set_transient_for(Some(&window));
            } else {
                tracing::error!("Couldn't find root of transient widget");
            }
        }));

        self
    }
}

impl<C: AsyncComponent> AsyncComponentBuilder<C>
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
    /// you should use [`transient_for`][AsyncComponentBuilder::transient_for] instead.
    #[must_use]
    pub fn transient_for_native(self, widget: impl AsRef<gtk::Widget>) -> Self {
        let widget = widget.as_ref().clone();
        let root = self.root.clone();
        late_initialization::register_callback(Box::new(move || {
            if let Some(window) = widget.toplevel_window() {
                root.as_ref().set_transient_for(Some(&window));
            } else {
                tracing::error!("Couldn't find root of transient widget");
            }
        }));

        self
    }
}

impl<C: AsyncComponent> AsyncComponentBuilder<C> {
    /// Starts the component, passing ownership to a future attached to a [gtk::glib::MainContext].
    pub fn launch(self, payload: C::Init) -> AsyncConnector<C> {
        // Used for all events to be processed by this component's internal service.
        let (input_sender, input_receiver) = crate::channel::<C::Input>();

        self.launch_with_input_channel(payload, input_sender, input_receiver)
    }

    /// Similar to [`launch()`](AsyncComponentBuilder::launch) but also initializes a [`MessageBroker`].
    ///
    /// # Panics
    ///
    /// This method panics if the message broker was already initialized in another launch.
    pub fn launch_with_broker(
        self,
        payload: C::Init,
        broker: &MessageBroker<C::Input>,
    ) -> AsyncConnector<C> {
        let (input_sender, input_receiver) = broker.get_channel();
        self.launch_with_input_channel(
            payload,
            input_sender,
            input_receiver.expect("Message broker launched multiple times"),
        )
    }

    fn launch_with_input_channel(
        self,
        payload: C::Init,
        input_sender: Sender<C::Input>,
        input_receiver: Receiver<C::Input>,
    ) -> AsyncConnector<C> {
        let Self {
            mut root, priority, ..
        } = self;
        let temp_widgets = C::init_loading_widgets(&mut root);

        let RuntimeSenders {
            output_sender,
            output_receiver,
            cmd_sender,
            cmd_receiver,
            shutdown_notifier,
            shutdown_recipient,
            shutdown_on_drop: destroy_on_drop,
            mut shutdown_event,
        } = RuntimeSenders::<C::Output, C::CommandOutput>::new();

        // Encapsulates the senders used by component methods.
        let component_sender = AsyncComponentSender::new(
            input_sender.clone(),
            output_sender.clone(),
            cmd_sender,
            shutdown_recipient,
        );

        let (source_id_sender, source_id_receiver) =
            oneshot::channel::<gtk::glib::JoinHandle<()>>();

        let rt_root = root.clone();

        // Spawns the component's service. It will receive both `Self::Input` and
        // `Self::CommandOutput` messages. It will spawn commands as requested by
        // updates, and send `Self::Output` messages externally.
        let handle = crate::spawn_local_with_priority(priority, async move {
            let id = source_id_receiver.await.unwrap().into_source_id().unwrap();
            let mut state = C::init(payload, rt_root.clone(), component_sender.clone()).await;
            drop(temp_widgets);

            let mut cmd = GuardedReceiver::new(cmd_receiver);
            let mut input = GuardedReceiver::new(input_receiver);

            loop {
                futures::select!(
                    // Performs the model update, checking if the update requested a command.
                    // Runs that command asynchronously in the background using tokio.
                    message = input => {
                        let AsyncComponentParts {
                            model,
                            widgets,
                        } = &mut state;

                        let span = info_span!(
                            "update_with_view",
                            input=?message,
                            component=any::type_name::<C>(),
                            id=model.id(),
                        );
                        let _enter = span.enter();

                        model.update_with_view(widgets, message, component_sender.clone(), &rt_root).await;
                    }

                    // Handles responses from a command.
                    message = cmd => {
                        let AsyncComponentParts {
                            model,
                            widgets,
                        } = &mut state;

                        let span = info_span!(
                            "update_cmd_with_view",
                            cmd_output=?message,
                            component=any::type_name::<C>(),
                            id=model.id(),
                        );
                        let _enter = span.enter();

                        model.update_cmd_with_view(widgets, message, component_sender.clone(), &rt_root).await;
                    }

                    // Triggered when the component is destroyed
                    _ = shutdown_event => {
                        let AsyncComponentParts {
                            model,
                            widgets,
                        } = &mut state;

                        model.shutdown(widgets, output_sender);

                        shutdown_notifier.shutdown();

                        id.remove();

                        return;
                    }
                );
            }
        });

        source_id_sender.send(handle).unwrap();

        // Give back a type for controlling the component service.
        AsyncConnector {
            widget: root,
            sender: input_sender,
            receiver: output_receiver,
            shutdown_on_drop: destroy_on_drop,
        }
    }
}
