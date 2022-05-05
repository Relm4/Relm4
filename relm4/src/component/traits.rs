// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::*;
use crate::{ComponentSender, Sender};

/// Elm-style variant of a Component with view updates separated from input updates
pub trait Component: Sized + 'static {
    /// Messages which are received from commands executing in the background.
    type CommandOutput: 'static + Send;

    /// The message type that the component accepts as inputs.
    type Input: 'static;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The initial parameter(s) for launch.
    type InitParams;

    /// The widget that was constructed by the component.
    type Root: std::fmt::Debug;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Create a builder for this component.
    fn builder() -> ComponentBuilder<Self> {
        ComponentBuilder::<Self>::new()
    }

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Creates the initial model and view, docking it into the component.
    fn init(
        params: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {}

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, message: Self::CommandOutput, sender: &ComponentSender<Self>) {}

    /// Handles updates from a command.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: &ComponentSender<Self>,
    ) {
        self.update_cmd(message, sender);
        self.update_view(widgets, sender)
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: &ComponentSender<Self>) {}

    /// Updates the model and view. Optionally returns a command to run.
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: &ComponentSender<Self>,
    ) {
        self.update(message, sender);
        self.update_view(widgets, sender);
    }

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}
}

/// Elm-style variant of a Component with view updates separated from input updates
pub trait SimpleComponent: Sized + 'static {
    /// The message type that the component accepts as inputs.
    type Input: 'static;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The initial parameter(s) for launch.
    type InitParams;

    /// The widget that was constructed by the component.
    type Root: std::fmt::Debug;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Creates the initial model and view, docking it into the component.
    fn init(
        params: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {}

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, input: &Sender<Self::Input>, output: &Sender<Self::Output>) {}

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(&self, widgets: &mut Self::Widgets, sender: &ComponentSender<Self>) {}

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}
}

impl<C> Component for C
where
    C: SimpleComponent,
{
    type InitParams = C::InitParams;
    type Input = C::Input;
    type Output = C::Output;
    type Root = C::Root;
    type Widgets = C::Widgets;

    type CommandOutput = ();

    fn init_root() -> Self::Root {
        C::init_root()
    }

    fn init(
        params: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        C::init(params, root, sender)
    }

    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {
        C::update(self, message, sender);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: &ComponentSender<Self>) {
        C::update_view(self, widgets, sender)
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {
        self.shutdown(widgets, output);
    }
}
