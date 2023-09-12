//! Components are smaller mostly independent parts of
//! your application.

/// Types and traits used for regular (synchronous) components.
mod sync;

/// Types and traits used for async components.
mod r#async;

/// Message broker
mod message_broker;

/// A simpler version of components that does work
/// in the background.
pub mod worker;

pub use message_broker::MessageBroker;

pub use sync::{
    CommandFuture, Component, ComponentBuilder, ComponentController, ComponentParts,
    ComponentStream, Connector, Controller, SimpleComponent, StateWatcher,
};

pub use r#async::{
    AsyncComponent, AsyncComponentBuilder, AsyncComponentController, AsyncComponentParts,
    AsyncComponentStream, AsyncConnector, AsyncController, SimpleAsyncComponent,
};

pub use crate::channel::AsyncComponentSender;
