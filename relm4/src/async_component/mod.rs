mod builder;
mod connector;
mod controller;
mod destroy_on_drop;
mod state_watcher;
mod traits;

pub use crate::sender::AsyncComponentSender;
pub use builder::AsyncComponentBuilder;
pub use connector::AsyncConnector;
pub use controller::{AsyncComponentController, AsyncController};
pub use state_watcher::StateWatcher;
pub use traits::AsyncComponent;
pub use traits::SimpleAsyncComponent;

/// Contains the initial model and widgets being docked into a component.
#[derive(Debug)]
pub struct AsyncComponentParts<C: AsyncComponent> {
    /// The model of the component.
    pub model: C,
    /// The widgets created for the view.
    pub widgets: C::Widgets,
}
