// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

// Prevent false positive in nightly
#![allow(unused_qualifications)]

use std::fmt::Debug;

use crate::channel::{AsyncComponentSender, Sender};
use crate::loading_widgets::LoadingWidgets;

use super::{AsyncComponentBuilder, AsyncComponentParts};

/// Asynchronous variant of [`Component`](crate::Component).
///
/// `AsyncComponent` is powerful and flexible, but for many use-cases the [`SimpleAsyncComponent`]
/// convenience trait will suffice.
pub trait AsyncComponent: Sized + 'static {
    /// Messages which are received from commands executing in the background.
    type CommandOutput: Debug + Send + 'static;

    /// The message type that the component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the component.
    type Init;

    /// The top-level widget of the component.
    type Root: Debug + Clone;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Create a builder for this component.
    #[must_use]
    fn builder() -> AsyncComponentBuilder<Self> {
        AsyncComponentBuilder::<Self>::default()
    }

    /// Initializes the root widget
    #[must_use]
    fn init_root() -> Self::Root;

    /// Allows you to initialize the root widget with a temporary value
    /// as a placeholder until the [`init()`](AsyncComponent::init)
    /// future completes.
    ///
    /// This method does nothing by default.
    #[must_use]
    fn init_loading_widgets(_root: Self::Root) -> Option<LoadingWidgets> {
        None
    }

    /// Creates the initial model and view, docking it into the component.
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> impl std::future::Future<Output = AsyncComponentParts<Self>>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Updates the model and view upon completion of a command.
    ///
    /// Overriding this method is helpful if you need access to the widgets while processing a
    /// command output.
    ///
    /// The default implementation of this method calls [`update_cmd`] followed by [`update_view`].
    /// If you override this method while using the [`component`] macro, you must remember to call
    /// [`update_view`] in your implementation. Otherwise, the view will not reflect the updated
    /// model.
    ///
    /// [`update_cmd`]: Self::update_cmd
    /// [`update_view`]: Self::update_view
    /// [`component`]: relm4_macros::component
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) -> impl std::future::Future<Output = ()> {
        async {
            self.update_cmd(message, sender.clone(), root).await;
            self.update_view(widgets, sender);
        }
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: AsyncComponentSender<Self>) {}

    /// Updates the model and view when a new input is received.
    ///
    /// Overriding this method is helpful if you need access to the widgets while processing an
    /// input.
    ///
    /// The default implementation of this method calls [`update`] followed by [`update_view`]. If
    /// you override this method while using the [`component`] macro, you must remember to
    /// call [`update_view`] in your implementation. Otherwise, the view will not reflect the
    /// updated model.
    ///
    /// [`update`]: Self::update
    /// [`update_view`]: Self::update_view
    /// [`component`]: relm4_macros::component
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) -> impl std::future::Future<Output = ()> {
        async {
            self.update(message, sender.clone(), root).await;
            self.update_view(widgets, sender);
        }
    }

    /// Last method called before a component is shut down.
    ///
    /// This method is guaranteed to be called even when the entire application is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}

    /// An identifier for the component used for debug logging.
    ///
    /// The default implementation of this method uses the address of the component, but
    /// implementations are free to provide more meaningful identifiers.
    #[must_use]
    fn id(&self) -> String {
        format!("{:p}", &self)
    }
}

/// Asynchronous variant of [`SimpleComponent`](crate::SimpleComponent).
pub trait SimpleAsyncComponent: Sized + 'static {
    /// The message type that the component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the component.
    type Init;

    /// The top-level widget of the component.
    type Root: Debug + Clone;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    #[must_use]
    fn init_root() -> Self::Root;

    /// Allows you to initialize the root widget with a temporary value
    /// as a placeholder until the [`init()`](AsyncComponent::init)
    /// future completes.
    ///
    /// This method does nothing by default.
    #[must_use]
    fn init_loading_widgets(_root: Self::Root) -> Option<LoadingWidgets> {
        None
    }

    /// Creates the initial model and view, docking it into the component.
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> impl std::future::Future<Output = AsyncComponentParts<Self>>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(
        &mut self,
        input: &Sender<Self::Input>,
        output: Sender<Self::Output>,
    ) -> impl std::future::Future<Output = ()> {
        async {}
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: AsyncComponentSender<Self>) {}

    /// Last method called before a component is shut down.
    ///
    /// This method is guaranteed to be called even when the entire application is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}
}

impl<C> AsyncComponent for C
where
    C: SimpleAsyncComponent,
{
    type Init = C::Init;
    type Input = C::Input;
    type Output = C::Output;
    type Root = C::Root;
    type Widgets = C::Widgets;

    type CommandOutput = ();

    fn init_root() -> Self::Root {
        C::init_root()
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        C::init_loading_widgets(root)
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        C::init(init, root, sender).await
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        C::update(self, message, sender).await;
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: AsyncComponentSender<Self>) {
        C::update_view(self, widgets, sender);
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {
        self.shutdown(widgets, output);
    }
}

/// An empty, non-interactive component as a placeholder for tests.
impl SimpleAsyncComponent for () {
    type Input = ();
    type Output = ();
    type Init = ();
    type Root = ();
    type Widgets = ();

    fn init_root() -> Self::Root {}

    async fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        AsyncComponentParts {
            model: (),
            widgets: (),
        }
    }
}
