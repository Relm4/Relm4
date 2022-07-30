// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use async_oneshot::oneshot;
use futures::FutureExt;
use gtk::glib;
use tracing::info_span;

use crate::component::{ComponentSenderInner, EmptyRoot};
use crate::{
    shutdown, Component, ComponentBuilder, ComponentParts, OnDestroy, Receiver, Sender,
    SimpleComponent,
};
use std::fmt::Debug;
use std::sync::Arc;
use std::{any, thread};

/// Receives inputs and outputs in the background.
pub trait Worker: Sized + Send + 'static {
    /// The initial parameters that will be used to build the worker state.
    type InitParams: 'static + Send;
    /// The type of inputs that this worker shall receive.
    type Input: 'static + Send + Debug;
    /// The typue of outputs that this worker shall send.
    type Output: 'static + Send + Debug;

    /// Defines the initial state of the worker.
    fn init(params: Self::InitParams, sender: crate::ComponentSender<Self>) -> Self;

    /// Defines how inputs will bep processed
    fn update(&mut self, message: Self::Input, sender: crate::ComponentSender<Self>);
}

impl<T> SimpleComponent for T
where
    T: Worker + 'static,
{
    type Root = EmptyRoot;
    type Widgets = ();

    type InitParams = <Self as Worker>::InitParams;
    type Input = <Self as Worker>::Input;
    type Output = <Self as Worker>::Output;

    fn init_root() -> Self::Root {
        EmptyRoot::default()
    }

    fn init(
        params: Self::InitParams,
        _root: &Self::Root,
        sender: crate::ComponentSender<Self>,
    ) -> crate::ComponentParts<Self> {
        let model = Self::init(params, sender);
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: crate::ComponentSender<Self>) {
        Self::update(self, message, sender);
    }
}

impl<C> ComponentBuilder<C>
where
    C: Component<Root = EmptyRoot, Widgets = ()> + Send,
    C::Input: Send,
    C::Output: Send,
    C::CommandOutput: Send,
{
    /// Starts a worker on a separate thread,
    /// passing ownership to a future attached to a GLib context.
    pub fn detach_worker(self, payload: C::InitParams) -> WorkerHandle<C> {
        let ComponentBuilder { root, .. } = self;

        // Used for all events to be processed by this component's internal service.
        let (input_tx, input_rx) = crate::channel::<C::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (output_tx, output_rx) = crate::channel::<C::Output>();

        // Sends messages from commands executed from the background.
        let (cmd_tx, cmd_rx) = crate::channel::<C::CommandOutput>();

        // Notifies the component's child commands that it is now deceased.
        let (death_notifier, death_recipient) = shutdown::channel();

        // Encapsulates the senders used by component methods.
        let component_sender = Arc::new(ComponentSenderInner {
            command: cmd_tx,
            input: input_tx.clone(),
            output: output_tx.clone(),
            shutdown: death_recipient,
        });

        // The source ID of the component's service will be sent through this once the root
        // widget has been iced, which will give the component one last chance to say goodbye.
        let (mut burn_notifier, burn_recipient) = oneshot::<()>();

        let mut state = C::init(payload, &root, component_sender.clone());

        thread::spawn(move || {
            let context =
                glib::MainContext::thread_default().unwrap_or_else(glib::MainContext::new);

            // Spawns the component's service. It will receive both `Self::Input` and
            // `Self::CommandOutput` messages. It will spawn commands as requested by
            // updates, and send `Self::Output` messages externally.
            context.block_on(async move {
                let mut burn_notice = burn_recipient.fuse();
                loop {
                    let cmd = cmd_rx.recv().fuse();
                    let input = input_rx.recv().fuse();

                    futures::pin_mut!(cmd);
                    futures::pin_mut!(input);

                    futures::select!(
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        message = input => {
                            if let Some(message) = message {
                                let &mut ComponentParts {
                                    ref mut model,
                                    ref mut widgets,
                                } = &mut state;

                                let span = info_span!(
                                    "update_with_view",
                                    input=?message,
                                    component=any::type_name::<C>(),
                                    id=model.id(),
                                );
                                let _enter = span.enter();

                                model.update_with_view(widgets, message, component_sender.clone());
                            }
                        }

                        // Handles responses from a command.
                        message = cmd => {
                            if let Some(message) = message {
                                let &mut ComponentParts {
                                    ref mut model,
                                    ref mut widgets,
                                } = &mut state;

                                let span = info_span!(
                                    "update_cmd_with_view",
                                    cmd_output=?message,
                                    component=any::type_name::<C>(),
                                    id=model.id(),
                                );
                                let _enter = span.enter();

                                model.update_cmd_with_view(widgets, message, component_sender.clone());
                            }
                        },

                        // Triggered when the component is destroyed
                        _ = burn_notice => {
                            let ComponentParts {
                                ref mut model,
                                ref mut widgets,
                            } = &mut state;

                            model.shutdown(widgets, output_tx);

                            death_notifier.shutdown();

                            return
                        }
                    );
                }
            });
        });

        // When the root widget is destroyed, the spawned service will be removed.
        root.on_destroy(move || {
            let _ = burn_notifier.send(());
        });

        // Give back a type for controlling the component service.
        WorkerHandle {
            sender: input_tx,
            receiver: output_rx,
            _root: root,
        }
    }
}

#[derive(Debug)]
/// Handle to a worker task in the background
pub struct WorkerHandle<W: Component> {
    // Sends inputs to the worker.
    sender: Sender<W::Input>,
    // Where the worker will send its outputs to.
    receiver: Receiver<W::Output>,
    _root: EmptyRoot,
}

impl<W: Component> WorkerHandle<W>
where
    W::Input: 'static,
    W::Output: 'static,
{
    /// Given a mutable closure, captures the receiver for handling.
    pub fn connect_receiver<F: FnMut(&mut Sender<W::Input>, W::Output) + 'static>(
        self,
        mut func: F,
    ) -> WorkerController<W> {
        let WorkerHandle {
            sender,
            receiver,
            _root,
        } = self;

        let mut sender_ = sender.clone();
        crate::spawn_local(async move {
            while let Some(event) = receiver.recv().await {
                func(&mut sender_, event);
            }
        });

        WorkerController { sender, _root }
    }

    /// Forwards output events to the designated sender.
    pub fn forward<X: 'static, F: (Fn(W::Output) -> X) + 'static>(
        self,
        sender: &Sender<X>,
        transform: F,
    ) -> WorkerController<W> {
        let WorkerHandle {
            sender: own_sender,
            receiver,
            _root,
        } = self;

        crate::spawn_local(receiver.forward(sender.clone(), transform));
        WorkerController {
            sender: own_sender,
            _root,
        }
    }

    /// Ignore outputs from the component and take the handle.
    pub fn detach(self) -> WorkerController<W> {
        let Self { sender, _root, .. } = self;

        WorkerController { sender, _root }
    }
}

/// Sends inputs to a worker. On drop, shuts down the worker.
#[derive(Debug)]
pub struct WorkerController<W: Component> {
    // Sends inputs to the worker.
    sender: Sender<W::Input>,
    _root: EmptyRoot,
}

impl<W: Component> WorkerController<W> {
    /// Emits an input to the component.
    pub fn emit(&self, event: W::Input) {
        self.sender.send(event);
    }

    /// Provides access to the component's sender.
    pub fn sender(&self) -> &Sender<W::Input> {
        &self.sender
    }
}
