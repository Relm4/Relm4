/// Types and traits used for regular (synchronous) components.
mod sync;

/// Types and traits used for async components.
mod r#async;

/// A simpler version of components that does work
/// in the background.
pub mod worker;

pub use sync::{
    CommandFuture, Component, ComponentBuilder, ComponentController, ComponentParts,
    ComponentStream, Connector, Controller, MessageBroker, SimpleComponent, StateWatcher,
};

pub use r#async::{
    AsyncComponent, AsyncComponentBuilder, AsyncComponentController, AsyncComponentParts,
    AsyncConnector, AsyncController, SimpleAsyncComponent,
};

pub use crate::sender::AsyncComponentSender;
