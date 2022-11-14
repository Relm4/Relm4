//! Defines traits and data types to generate widgets from collections efficiently.

mod dynamic_index;

/// Traits and implementations used for factories to interact with widgets.
pub mod widgets;

/// Implementation of asynchronous factories.
mod r#async;
pub mod positions;

/// Implementation of regular factories.
mod sync;

pub use r#async::{AsyncFactoryComponent, AsyncFactoryVecDeque, AsyncFactoryVecDequeGuard};
pub use sync::{FactoryComponent, FactoryVecDeque, FactoryVecDequeGuard};

pub use crate::sender::{AsyncFactoryComponentSender, FactoryComponentSender};
pub use dynamic_index::DynamicIndex;
pub use widgets::traits::*;
