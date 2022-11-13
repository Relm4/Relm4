// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

//! Contains various flavors of channels to send messages between components and workers.

use std::future::Future;
use std::sync::Arc;

use crate::component::AsyncComponent;
use crate::factory::FactoryComponent;
use crate::{Component, Sender, ShutdownReceiver};

/// Contains senders to send and receive messages from a [`Component`].
#[derive(Debug)]
pub struct ComponentSender<C: Component> {
    shared: Arc<ComponentSenderInner<C::Input, C::Output, C::CommandOutput>>,
}

impl<C: Component> ComponentSender<C> {
    pub(crate) fn new(
        input_tx: Sender<C::Input>,
        output_tx: Sender<C::Output>,
        command_tx: Sender<C::CommandOutput>,
        shutdown_tx: ShutdownReceiver,
    ) -> Self {
        Self {
            shared: Arc::new(ComponentSenderInner {
                input: input_tx,
                output: output_tx,
                command: command_tx,
                shutdown: shutdown_tx,
            }),
        }
    }

    /// Spawns a command.
    /// You can bind the the command to the lifetime of the component
    /// by using a [`ShutdownReceiver`].
    pub fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: FnOnce(Sender<C::CommandOutput>, ShutdownReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        self.shared.command(cmd);
    }

    /// Spawns a future that will be dropped as soon as the component is shut down.
    ///
    /// Essentially, this is a simpler version of [`Self::command()`].
    pub fn oneshot_command<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = C::CommandOutput> + Send + 'static,
    {
        self.shared.oneshot_command(future);
    }

    /// Emit an input to the component.
    pub fn input(&self, message: C::Input) {
        self.shared.input.send(message);
    }

    /// Retrieve the sender for input messages.
    ///
    /// Useful to forward inputs from another component. If you just need to send input messages,
    /// [`input`][Self::input] is more concise.
    #[must_use]
    pub fn input_sender(&self) -> &Sender<C::Input> {
        &self.shared.input
    }

    /// Emit an output to the component.
    pub fn output(&self, message: C::Output) {
        self.shared.output.send(message);
    }

    /// Retrieve the sender for output messages.
    ///
    /// Useful to forward outputs from another component. If you just need to send output messages,
    /// [`output`][Self::output] is more concise.
    #[must_use]
    pub fn output_sender(&self) -> &Sender<C::Output> {
        &self.shared.output
    }
}

impl<C: Component> Clone for ComponentSender<C> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

/// Contain senders to send and receive messages from a [`FactoryComponent`].
#[derive(Debug)]
pub struct FactoryComponentSender<C: FactoryComponent> {
    shared: Arc<ComponentSenderInner<C::Input, C::Output, C::CommandOutput>>,
}

impl<C: FactoryComponent> FactoryComponentSender<C> {
    pub(crate) fn new(
        input_tx: Sender<C::Input>,
        output_tx: Sender<C::Output>,
        command_tx: Sender<C::CommandOutput>,
        shutdown_tx: ShutdownReceiver,
    ) -> Self {
        Self {
            shared: Arc::new(ComponentSenderInner {
                input: input_tx,
                output: output_tx,
                command: command_tx,
                shutdown: shutdown_tx,
            }),
        }
    }

    /// Spawns a command.
    /// You can bind the the command to the lifetime of the component
    /// by using a [`ShutdownReceiver`].
    pub fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: FnOnce(Sender<C::CommandOutput>, ShutdownReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        self.shared.command(cmd);
    }

    /// Spawns a future that will be dropped as soon as the component is shut down.
    ///
    /// Essentially, this is a simpler version of [`Self::command()`].
    pub fn oneshot_command<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = C::CommandOutput> + Send + 'static,
    {
        self.shared.oneshot_command(future);
    }

    /// Emit an input to the component.
    pub fn input(&self, message: C::Input) {
        self.shared.input.send(message);
    }

    /// Retrieve the sender for input messages.
    ///
    /// Useful to forward inputs from another component. If you just need to send input messages,
    /// [`input`][Self::input] is more concise.
    #[must_use]
    pub fn input_sender(&self) -> &Sender<C::Input> {
        &self.shared.input
    }

    /// Emit an output to the component.
    pub fn output(&self, message: C::Output) {
        self.shared.output.send(message);
    }

    /// Retrieve the sender for output messages.
    ///
    /// Useful to forward outputs from another component. If you just need to send output messages,
    /// [`output`][Self::output] is more concise.
    #[must_use]
    pub fn output_sender(&self) -> &Sender<C::Output> {
        &self.shared.output
    }
}

impl<C: FactoryComponent> Clone for FactoryComponentSender<C> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

// Contains senders used by the component.
#[derive(Debug)]
struct ComponentSenderInner<Input, Output, Cmd> {
    /// Emits command outputs
    command: Sender<Cmd>,

    /// Emits component inputs
    input: Sender<Input>,

    /// Emits component outputs
    output: Sender<Output>,

    shutdown: ShutdownReceiver,
}

impl<Input, Output, CommandOutput: Send + 'static>
    ComponentSenderInner<Input, Output, CommandOutput>
{
    fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: FnOnce(Sender<CommandOutput>, ShutdownReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let recipient = self.shutdown.clone();
        let sender = self.command.clone();
        crate::spawn(async move {
            cmd(sender, recipient).await;
        });
    }

    fn oneshot_command<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = CommandOutput> + Send + 'static,
    {
        // Async closures would be awesome here...
        self.command(move |out, shutdown| {
            shutdown
                .register(async move { out.send(future.await) })
                .drop_on_shutdown()
        });
    }
}

/// Contains senders to send and receive messages from a [`Component`].
#[derive(Debug)]
pub struct AsyncComponentSender<C: AsyncComponent> {
    shared: Arc<ComponentSenderInner<C::Input, C::Output, C::CommandOutput>>,
}

impl<C: AsyncComponent> AsyncComponentSender<C> {
    pub(crate) fn new(
        input_tx: Sender<C::Input>,
        output_tx: Sender<C::Output>,
        command_tx: Sender<C::CommandOutput>,
        shutdown_tx: ShutdownReceiver,
    ) -> Self {
        Self {
            shared: Arc::new(ComponentSenderInner {
                input: input_tx,
                output: output_tx,
                command: command_tx,
                shutdown: shutdown_tx,
            }),
        }
    }

    /// Spawns a command.
    /// You can bind the the command to the lifetime of the component
    /// by using a [`ShutdownReceiver`].
    pub fn command<Cmd, Fut>(&self, cmd: Cmd)
    where
        Cmd: FnOnce(Sender<C::CommandOutput>, ShutdownReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        self.shared.command(cmd);
    }

    /// Spawns a future that will be dropped as soon as the component is shut down.
    ///
    /// Essentially, this is a simpler version of [`Self::command()`].
    pub fn oneshot_command<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = C::CommandOutput> + Send + 'static,
    {
        self.shared.oneshot_command(future);
    }

    /// Emit an input to the component.
    pub fn input(&self, message: C::Input) {
        self.shared.input.send(message);
    }

    /// Retrieve the sender for input messages.
    ///
    /// Useful to forward inputs from another component. If you just need to send input messages,
    /// [`input`][Self::input] is more concise.
    #[must_use]
    pub fn input_sender(&self) -> &Sender<C::Input> {
        &self.shared.input
    }

    /// Emit an output to the component.
    pub fn output(&self, message: C::Output) {
        self.shared.output.send(message);
    }

    /// Retrieve the sender for output messages.
    ///
    /// Useful to forward outputs from another component. If you just need to send output messages,
    /// [`output`][Self::output] is more concise.
    #[must_use]
    pub fn output_sender(&self) -> &Sender<C::Output> {
        &self.shared.output
    }
}

impl<C: AsyncComponent> Clone for AsyncComponentSender<C> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}
