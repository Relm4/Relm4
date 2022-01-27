// Copyright 2021-2022 Aaron Erhardt <aaron.erhardt@t-online.de>
// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod finalized;
mod handle;

#[allow(unreachable_pub)]
pub use self::finalized::Finalized;
#[allow(unreachable_pub)]
pub use self::handle::Handle;

use crate::{Receiver, Sender};
use gtk::prelude::*;
use tokio::sync::mpsc;

#[async_trait::async_trait]
/// Elm-style variant of a Component with view updates separated from input updates
pub trait Component: Sized + 'static {
    /// Internal commands to perform
    type Command: 'static + Send;

    /// The arguments that are passed to the init_view method.
    type InitParams;

    /// The message type that the component accepts as inputs.
    type Input: 'static + Send;

    /// The message type that the component provides as outputs.
    type Output: 'static;

    /// The widget that was constructed by the component.
    type Root: Clone + AsRef<gtk::Widget>;

    /// The type that's used for storing widgets created for this component.
    type Widgets: 'static;

    /// Initializes the root widget
    fn init_root() -> Self::Root;

    /// Initializes the model and view using the initial parameters.
    fn init_inner(
        params: Self::InitParams,
        root: &Self::Root,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets>;

    /// Initializes the component and attaches it to the thread-local GLib executor.
    ///
    /// The context's service remains alive for as long as the root widget is alive.
    fn init(
        params: Self::InitParams,
    ) -> RawComponent<Self, Self::Root, Self::Widgets, Self::Input, Self::Output> {
        let root = Self::init_root();

        // Used for all events to be processed by this component's internal service.
        let (mut input_tx, input_rx) = mpsc::unbounded_channel::<Self::Input>();

        // Used by this component to send events to be handled externally by the caller.
        let (mut output_tx, output_rx) = mpsc::unbounded_channel::<Self::Output>();

        // This channel enables forwards inputs to the internal service.
        let (inner_tx, inner_rx) = mpsc::unbounded_channel::<InnerMessage<Self::Input>>();

        // Send a `Drop` message to the internal service.
        root.as_ref()
            .connect_destroy(gtk::glib::clone!(@strong inner_tx => move |_| {
                let _ = inner_tx.send(InnerMessage::Drop);
            }));

        // Constructs the initial model and view with the initial params.
        let ComponentParts { model, widgets } =
            Self::init_inner(params, &root, &mut input_tx, &mut output_tx);

        RawComponent {
            model,
            widgets,
            root,
            input: input_tx,
            output: output_tx,
            input_rx,
            inner_tx,
            inner_rx,
            output_rx,
        }
    }

    /// Handles input messages and enables the programmer to update the model and view.
    fn update(
        &mut self,
        message: Self::Input,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command>;

    /// Update the UI
    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    );

    /// A command to perform in a background thread.
    async fn command(message: Self::Command) -> Option<Self::Input>;
}

/// A component which has been initialized, but hasn't been finalized yet.
#[derive(Debug)]
pub struct RawComponent<Model, Root, Widgets, Input, Output> {
    /// The model of the component.
    pub model: Model,
    /// The root widget of the component.
    pub root: Root,
    /// The widgets created for the view.
    pub widgets: Widgets,
    /// Sender for inputs to the component.
    pub input: Sender<Input>,
    output: Sender<Output>,
    input_rx: Receiver<Input>,
    inner_tx: Sender<InnerMessage<Input>>,
    inner_rx: Receiver<InnerMessage<Input>>,
    output_rx: Receiver<Output>,
}

impl<C: Component> RawComponent<C, C::Root, C::Widgets, C::Input, C::Output> {
    /// Starts the component, passing ownership to a future attached to a GLib context.
    pub fn finalize(self) -> Finalized<C::Root, C::Input, C::Output> {
        let RawComponent {
            inner_tx,
            input_rx,
            input,
            mut inner_rx,
            mut model,
            mut output,
            mut widgets,
            output_rx,
            root,
        } = self;

        // Forward `Self::Input` events to the internal service.
        let forwarder = crate::forward(input_rx, inner_tx, InnerMessage::Message);

        // The internal service which manages all input requests.
        let mut input_ = input.clone();
        let service = async move {
            while let Some(event) = inner_rx.recv().await {
                match event {
                    InnerMessage::Message(event) => {
                        // Performs the model update, checking if the update requested a command.
                        // Runs that command asynchronously in the background using tokio.
                        if let Some(command) = model.update(event, &mut input_, &mut output) {
                            let input = input_.clone();
                            tokio::spawn(async move {
                                if let Some(event) = C::command(command).await {
                                    let _ = input.send(event);
                                }
                            });
                        }

                        model.update_view(&mut widgets, &mut input_, &mut output);
                    }

                    InnerMessage::Drop => break,
                }
            }
        };

        // Start the internal service and its message forwarder.
        crate::spawn_local(async move {
            futures::future::join(forwarder, service).await;
        });

        Finalized {
            widget: root,
            sender: input,
            receiver: output_rx,
        }
    }
}

/// Constructed in `Component::init`.
#[derive(Debug)]
pub struct ComponentParts<Model, Widgets> {
    /// The model of the component.
    pub model: Model,
    /// The widgets created for the view.
    pub widgets: Widgets,
}

/// Used to drop the component's event loop when the managed widget is destroyed.
enum InnerMessage<T> {
    Drop,
    Message(T),
}
