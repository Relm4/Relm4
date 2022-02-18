// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use super::*;
use crate::Sender;
use std::marker::PhantomData;

/// Component with view updates happening at the same time as model updates.
pub trait StatefulComponent: Sized + 'static {
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
    type Root: OnDestroy;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Initializes the root widget and prepares a `Bridge` for docking.
    fn init() -> ComponentBuilder<Self, Self::Root> {
        ComponentBuilder {
            root: Self::init_root(),
            component: PhantomData,
        }
    }

    /// Creates the initial model and view, docking it into the component.
    fn init_parts(
        params: Self::InitParams,
        root: &Self::Root,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets>;

    /// Processes inputs received by the component.
    #[allow(unused)]
    fn update(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command> {
        None
    }

    /// Defines how the component should respond to command updates.
    #[allow(unused)]
    fn update_cmd(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
    }

    /// Called after a component's model has been updated externally.
    #[allow(unused)]
    fn update_notify(
        &mut self,
        widgets: &mut Self::Widgets,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
    }

    /// A command to perform in a background thread.
    #[allow(unused)]
    fn command(message: Self::Command) -> CommandFuture<Self::CommandOutput> {
        Box::pin(async move { None })
    }

    /// Last method called before a component is shut down.
    #[allow(unused)]
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}
}
