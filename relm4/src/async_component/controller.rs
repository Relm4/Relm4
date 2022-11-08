// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::fmt::{self, Debug};

use crate::Sender;

use super::{destroy_on_drop::DestroyOnDrop, AsyncComponent};

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
}

/// Controls the component from afar.
pub struct AsyncController<C: AsyncComponent> {
    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,

    /// Type used to destroy the async component when it's dropped.
    pub(super) destroy_sender: DestroyOnDrop,
}

impl<C: AsyncComponent> AsyncController<C> {
    /// Remove the controller but keep the runtime alive.
    pub fn detach_runtime(mut self) {
        self.destroy_sender.deactivate();
    }
}

impl<C: AsyncComponent> AsyncComponentController<C> for AsyncController<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn widget(&self) -> &C::Root {
        &self.widget
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
