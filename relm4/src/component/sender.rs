// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::{Component, Sender, ShutdownReceiver};
use std::future::Future;
use std::sync::Arc;

/// Contain senders used by the component.
pub type ComponentSender<C> = Arc<ComponentSenderInner<C>>;

/// Contains senders used by the component.
#[derive(Debug)]
pub struct ComponentSenderInner<C: Component> {
    /// Emits command outputs
    pub(crate) command: Sender<C::CommandOutput>,

    /// Emits component inputs
    pub input: Sender<C::Input>,

    /// Emits component outputs
    pub output: Sender<C::Output>,

    pub(crate) shutdown: ShutdownReceiver,
}

impl<C: Component> ComponentSenderInner<C> {
    /// Spawn a command managed by the lifetime of the component.
    pub fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: (Fn(Sender<C::CommandOutput>, ShutdownReceiver) -> Fut) + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let recipient = self.shutdown.clone();
        let sender = self.command.clone();
        crate::spawn(async move {
            cmd(sender, recipient).await;
        });
    }

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
