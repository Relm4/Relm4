// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod bridge;
mod controller;
mod elm_like;
mod fairing;
mod stateful;

#[allow(unreachable_pub)]
pub use self::bridge::Bridge;
#[allow(unreachable_pub)]
pub use self::controller::Controller;
#[allow(unreachable_pub)]
pub use self::elm_like::Component;
#[allow(unreachable_pub)]
pub use self::fairing::Fairing;
#[allow(unreachable_pub)]
pub use self::stateful::StatefulComponent;

use std::future::Future;
use std::pin::Pin;

/// A future returned by a component's command method.
pub type CommandFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct Fuselage<Model, Widgets> {
    /// The model of the component.
    pub model: Model,
    /// The widgets created for the view.
    pub widgets: Widgets,
}
