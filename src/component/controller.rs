// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::*;
use std::rc::Rc;

/// Shared behavior of component controller types.
pub trait ComponentController<C: Component> {
    /// Emits an input to the component.
    fn emit(&self, event: C::Input) {
        let _ = self.sender().send(event);
    }

    /// Provides access to the component's sender.
    fn sender(&self) -> &Sender<C::Input>;

    /// Provides access to the state of a component.
    fn state(&self) -> &Rc<StateWatcher<C>>;

    /// The root widget of the component.
    fn widget(&self) -> &C::Root;
}

/// Controls the component from afar.
#[allow(missing_debug_implementations)]
pub struct Controller<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: Rc<StateWatcher<C>>,

    /// The widget that this component manages.
    pub(super) widget: C::Root,

    /// Used for emitting events to the component.
    pub(super) sender: Sender<C::Input>,
}

impl<C: Component> ComponentController<C> for Controller<C> {
    fn sender(&self) -> &Sender<C::Input> {
        &self.sender
    }

    fn state(&self) -> &Rc<StateWatcher<C>> {
        &self.state
    }

    fn widget(&self) -> &C::Root {
        &self.widget
    }
}
