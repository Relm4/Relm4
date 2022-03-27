// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::*;
use crate::{shutdown::ShutdownReceiver, Sender};

/// Elm-style variant of a Component with view updates separated from input updates
pub trait Component: Sized + 'static {
    /// Internal commands to perform
    type Command: 'static + Send;

    /// Messages which are received from commands executing in the background.
    type CommandOutput: 'static + Send;

    /// The message type that the component accepts as inputs.
    type Input: 'static;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The initial parameter(s) for launch.
    type InitParams;

    /// The widget that was constructed by the component.
    type Root: std::fmt::Debug + OnDestroy;

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
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> Option<Self::Command> {
        None
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
    }

    /// Handles updates from a command.
    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
        self.update_cmd(message, input, output);
        self.update_view(widgets, input, output)
    }

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
    }

    /// Updates the model and view. Optionally returns a command to run.
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> Option<Self::Command> {
        let cmd = self.update(message, input, output);
        self.update_view(widgets, input, output);
        cmd
    }

    /// A command to perform in a background thread.
    #[allow(unused)]
    fn command(
        message: Self::Command,
        shutdown: ShutdownReceiver,
        output: Sender<Self::CommandOutput>,
    ) -> CommandFuture {
        Box::pin(async {})
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
    type Root: std::fmt::Debug + OnDestroy;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Creates the initial model and view, docking it into the component.
    fn init(
        params: Self::InitParams,
        root: &Self::Root,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> ComponentParts<Self>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        message: Self::Input,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(&mut self, input: &Sender<Self::Input>, output: &Sender<Self::Output>) {}

    /// Updates the view after the model has been updated.
    #[allow(unused)]
    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
    }

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

    type Command = ();
    type CommandOutput = ();

    fn init_root() -> Self::Root {
        C::init_root()
    }

    fn init(
        params: Self::InitParams,
        root: &Self::Root,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> ComponentParts<Self> {
        C::init(params, root, input, output)
    }

    fn update(
        &mut self,
        message: Self::Input,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> Option<Self::Command> {
        C::update(self, message, input, output);
        None
    }

    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) {
        C::update_view(self, widgets, input, output)
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {
        self.shutdown(widgets, output);
    }
}
