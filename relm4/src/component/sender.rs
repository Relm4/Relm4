// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::{Component, Sender};
use std::sync::Arc;

/// Contain senders used by the component.
pub type ComponentSender<C> = Arc<ComponentSenderInner<C>>;

/// Contains senders used by the component.
#[derive(Debug)]
pub struct ComponentSenderInner<C: Component> {
    /// Emits component inputs
    pub input: Sender<C::Input>,

    /// Emits component outputs
    pub output: Sender<C::Output>,
}

impl<C: Component> ComponentSenderInner<C> {
    /// Emit an input to the component.
    pub fn input(&self, message: C::Input) {
        self.input.send(message);
    }

    /// Equivalent to `&self.input`.
    pub fn input_sender(&self) -> &Sender<C::Input> {
        &self.input
    }

    /// Emit an output to the component.
    pub fn output(&self, message: C::Output) {
        self.output.send(message);
    }

    /// Equivalent to `&self.output`.
    pub fn output_sender(&self) -> &Sender<C::Output> {
        &self.output
    }
}
