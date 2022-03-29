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

    /// Stops the event loop when triggered.
    pub(crate) killswitch: flume::Sender<()>,

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

    /// Requests to stop this component's event loop.
    pub fn stop(&self) {
        let _ = self.killswitch.send(());
    }

    /// Provides access to the killswitch sender.
    pub fn stop_sender(&self) -> &flume::Sender<()> {
        &self.killswitch
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

// GTK integration
impl<C: Component> ComponentSenderInner<C> {
    /// Attaches the stop signal to a widget.
    pub fn stop_with_widget(self: &Arc<Self>, widget: &dyn AsRef<gtk::Widget>) {
        use gtk::prelude::WidgetExt;
        let this = self.clone();
        widget
            .as_ref()
            .connect_destroy(move |_| this.stop());
    }

    /// Attaches the stop signal to a native dialog
    pub fn stop_with_native_dialog(self: &Arc<Self>, dialog: &dyn AsRef<gtk::NativeDialog>) {
        use gtk::prelude::NativeDialogExt;
        let this = self.clone();
        dialog.as_ref().connect_response(move |_, response| {
            if let gtk::ResponseType::DeleteEvent = response {
                this.stop();
            }
        });
    }
}
