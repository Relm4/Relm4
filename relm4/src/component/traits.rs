// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::fmt::Debug;

use crate::{ComponentBuilder, ComponentParts, ComponentSender, OnDestroy, Sender};

/// Elm-style variant of a Component with view updates separated from input updates
pub trait Component: Sized + 'static {
    /// Messages which are received from commands executing in the background.
    type CommandOutput: Debug + Send + 'static;

    /// The message type that the component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the component.
    type Init;

    /// The widget that was constructed by the component.
    type Root: Debug + OnDestroy;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Create a builder for this component.
    #[must_use]
    fn builder() -> ComponentBuilder<Self> {
        ComponentBuilder::<Self>::default()
    }

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Creates the initial model and view, docking it into the component.
    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {}

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, message: Self::CommandOutput, sender: ComponentSender<Self>) {}

    /// Handles updates from a command.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
    ) {
        self.update_cmd(message, sender.clone());
        self.update_view(widgets, sender);
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {}

    /// Updates the model and view. Optionally returns a command to run.
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
    ) {
        self.update(message, sender.clone());
        self.update_view(widgets, sender);
    }

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}

    /// An identifier for the component used for debug logging.
    ///
    /// The default implementation of this method uses the address of the component, but
    /// implementations are free to provide more meaningful identifiers.
    fn id(&self) -> String {
        format!("{:p}", &self)
    }
}

/// Elm-style variant of a Component with view updates separated from input updates
pub trait SimpleComponent: Sized + 'static {
    /// The message type that the component accepts as inputs.
    type Input: Debug + 'static;

    /// The message type that the component provides as outputs.
    type Output: Debug + 'static;

    /// The parameter used to initialize the component.
    type Init;

    /// The widget that was constructed by the component.
    type Root: Debug + OnDestroy;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Creates the initial model and view, docking it into the component.
    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {}

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, input: &Sender<Self::Input>, output: Sender<Self::Output>) {}

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {}

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}
}

impl<C> Component for C
where
    C: SimpleComponent,
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

    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        C::init(params, root, sender)
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        C::update(self, message, sender);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        C::update_view(self, widgets, sender);
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {
        self.shutdown(widgets, output);
    }
}
