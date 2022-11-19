// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::cell::Ref;
use std::fmt::{self, Debug};

use crate::Sender;

use super::{Component, StateWatcher};

/// Shared behavior of component controller types.
pub trait ComponentController<C: Component> {
    /// Emits an input to the component.
    fn emit(&self, event: C::Input) {
        self.sender().send(event);
    }

    /// Provides access to the component's sender.
    fn sender(&self) -> &Sender<C::Input>;

    /// Provides access to the state of a component.
    fn state(&self) -> &StateWatcher<C>;

    /// Returns a reference to the [`Component`].
    fn model(&self) -> Ref<'_, C> {
        let part_ref = self.state().get();
        Ref::map(part_ref, |part| &part.model)
    }

    /// Returns a reference to the [`Component::Widgets`].
    fn widgets(&self) -> Ref<'_, C::Widgets> {
        let part_ref = self.state().get();
        Ref::map(part_ref, |part| &part.widgets)
    }

    /// Returns the root widget of the component.
    fn widget(&self) -> &C::Root;

    /// Dropping this type will usually stop the runtime of the component.
    /// With this method you can give the runtime a static lifetime.
    /// In other words, dropping the controller or connector will not stop
    /// the runtime anymore, instead it will run until the app is closed.
    fn detach_runtime(&mut self);
}

/// Controls the component from afar.
pub struct Controller<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: StateWatcher<C>,

    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,
}

impl<C: Component> ComponentController<C> for Controller<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn state(&self) -> &StateWatcher<C> {
        &self.state
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }

    fn detach_runtime(&mut self) {
        self.state.detach_runtime();
    }
}

impl<C> Debug for Controller<C>
where
    C: Component + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Controller")
            .field("state", &self.state)
            .field("widget", &self.widget)
            .field("sender", &self.sender)
            .finish()
    }
}
