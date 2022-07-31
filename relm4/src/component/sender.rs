// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use crate::{Component, Sender, ShutdownReceiver};
use std::future::Future;
use std::sync::Arc;

/// Contain senders used by the component.
pub type ComponentSender<C> = Arc<
    ComponentSenderInner<
        <C as Component>::Input,
        <C as Component>::Output,
        <C as Component>::CommandOutput,
    >,
>;

/// Contains senders used by the component.
#[derive(Debug)]
pub struct ComponentSenderInner<Input, Output, Cmd> {
    /// Emits command outputs
    pub(crate) command: Sender<Cmd>,

    /// Emits component inputs
    pub input: Sender<Input>,

    /// Emits component outputs
    pub output: Sender<Output>,

    pub(crate) shutdown: ShutdownReceiver,
}

impl<Input, Output, CommandOutput: Send + 'static>
    ComponentSenderInner<Input, Output, CommandOutput>
{
    /// Spawn a command managed by the lifetime of the component.
    pub fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: (FnOnce(Sender<CommandOutput>, ShutdownReceiver) -> Fut) + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let recipient = self.shutdown.clone();
        let sender = self.command.clone();
        crate::spawn(async move {
            cmd(sender, recipient).await;
        });
    }

    /// Emit an input to the component.
    pub fn input(&self, message: Input) {
        self.input.send(message);
    }

    /// Equivalent to `&self.input`.
    pub fn input_sender(&self) -> &Sender<Input> {
        &self.input
    }

    /// Emit an output to the component.
    pub fn output(&self, message: Output) {
        self.output.send(message);
    }

    /// Equivalent to `&self.output`.
    pub fn output_sender(&self) -> &Sender<Output> {
        &self.output
    }
}
