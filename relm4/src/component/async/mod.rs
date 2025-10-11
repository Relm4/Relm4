mod builder;
mod connector;
mod controller;
mod stream;
mod traits;
mod worker;

pub use builder::AsyncComponentBuilder;
pub use connector::AsyncConnector;
pub use controller::{AsyncComponentController, AsyncController};
pub use stream::AsyncComponentStream;
pub use traits::AsyncComponent;
pub use traits::SimpleAsyncComponent;
pub use worker::{AsyncWorker, AsyncWorkerController, AsyncWorkerHandle};

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct AsyncComponentParts<C: AsyncComponent> {
    /// The model of the component.
    pub model: C,
    /// The widgets created for the view.
    pub widgets: C::Widgets,
}
