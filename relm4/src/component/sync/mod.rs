// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod builder;
mod connector;
mod controller;
mod state_watcher;
mod stream;
mod traits;

pub use builder::ComponentBuilder;
pub use connector::Connector;
pub use controller::{ComponentController, Controller};
pub use state_watcher::StateWatcher;
pub use stream::ComponentStream;
pub use traits::{Component, SimpleComponent};

use std::future::Future;
use std::pin::Pin;

/// A future returned by a component's command method.
pub type CommandFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct ComponentParts<C: Component> {
    /// The model of the component.
    pub model: C,
    /// The widgets created for the view.
    pub widgets: C::Widgets,
}
