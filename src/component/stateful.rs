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

    /// The message type that the component accepts as inputs.
    type Input: 'static + Send;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The initial parameter(s) for launch.
    type Payload;

    /// The widget that was constructed by the component.
    type Root: OnDestroy;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Initializes the root widget and prepares a `Bridge` for docking.
    fn init() -> Bridge<Self, Self::Root> {
        Bridge {
            root: Self::init_root(),
            component: PhantomData,
        }
    }

    /// Creates the initial model and view, docking it into the component.
    fn dock(
        params: Self::Payload,
        root: &Self::Root,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Fuselage<Self, Self::Widgets>;

    /// Processes inputs received by the component.
    fn update(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command>;

    /// A command to perform in a background thread.
    #[allow(unused)]
    fn command(message: Self::Command, input: Sender<Self::Input>) -> CommandFuture {
        Box::pin(async move {})
    }
}
