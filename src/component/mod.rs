// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod bridge;
mod controller;
mod fairing;

#[allow(unreachable_pub)]
pub use self::bridge::Bridge;
#[allow(unreachable_pub)]
pub use self::controller::Controller;
#[allow(unreachable_pub)]
pub use self::fairing::Fairing;

use crate::Sender;
use std::marker::PhantomData;

#[async_trait::async_trait]
/// Elm-style variant of a Component with view updates separated from input updates
pub trait Component: Sized + 'static {
    /// Internal commands to perform
    type Command: 'static + Send;

    /// The message type that the component accepts as inputs.
    type Input: 'static + Send;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The initial parameter(s) for launch.
    type Payload;

    /// The widget that was constructed by the component.
    type Root: Clone + AsRef<gtk::Widget>;

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
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command>;

    /// Updates the view after the model has been updated.
    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    );

    /// A command to perform in a background thread.
    #[allow(unused)]
    async fn command(message: Self::Command, input: Sender<Self::Input>) {}
}

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct Fuselage<Model, Widgets> {
    /// The model of the component.
    pub model: Model,
    /// The widgets created for the view.
    pub widgets: Widgets,
}
