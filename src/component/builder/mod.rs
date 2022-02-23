// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod elm_like;
mod stateful;

use crate::RelmContainerExt;
use gtk::prelude::GtkWindowExt;
use std::marker::PhantomData;

/// A component that is ready for docking and launch.
#[derive(Debug)]
pub struct ComponentBuilder<Component, Root> {
    /// The root widget of the component.
    pub root: Root,

    pub(super) component: PhantomData<Component>,
}

impl<Component, Root> ComponentBuilder<Component, Root> {
    /// Configure the root widget before launching.
    pub fn update_root<F: FnOnce(&mut Root) + 'static>(mut self, func: F) -> Self {
        func(&mut self.root);
        self
    }
}

impl<Component, Root: AsRef<gtk::Widget>> ComponentBuilder<Component, Root> {
    /// Attach the component's root widget to a given container.
    pub fn attach_to(self, container: &impl RelmContainerExt) -> Self {
        container.container_add(self.root.as_ref());

        self
    }
}

impl<Component, Root: AsRef<gtk::Window>> ComponentBuilder<Component, Root> {
    /// Set the component's root widget transient for a given window.
    pub fn transient_for(self, window: impl AsRef<gtk::Window>) -> Self {
        self.root.as_ref().set_transient_for(Some(window.as_ref()));

        self
    }
}
