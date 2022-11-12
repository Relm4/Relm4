// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::fmt::{self, Debug};

use crate::Sender;

use super::AsyncComponent;
use crate::component::shutdown_on_drop::ShutdownOnDrop;

/// Shared behavior of component controller types.
pub trait AsyncComponentController<C: AsyncComponent> {
    /// Emits an input to the component.
    fn emit(&self, event: C::Input) {
        self.sender().send(event);
    }

    /// Provides access to the component's sender.
    fn sender(&self) -> &Sender<C::Input>;

    /// Returns the root widget of the component.
    fn widget(&self) -> &C::Root;

    /// Dropping this type will usually stop the runtime of the component.
    /// With this method you can give the runtime a static lifetime.
    /// In other words, dropping the controller or connector will not stop
    /// the runtime anymore, instead it will run until the app is closed.
    fn detach_runtime(&mut self);
}

/// Controls the component from afar.
pub struct AsyncController<C: AsyncComponent> {
    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,

    /// Type used to destroy the async component when it's dropped.
    pub(super) shutdown_on_drop: ShutdownOnDrop,
}

impl<C: AsyncComponent> AsyncComponentController<C> for AsyncController<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }

    fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate();
    }
}

impl<C> Debug for AsyncController<C>
where
    C: AsyncComponent + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Controller")
            .field("widget", &self.widget)
            .field("sender", &self.sender)
            .finish()
    }
}
