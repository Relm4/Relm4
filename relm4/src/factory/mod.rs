//! Defines traits and data types to generate widgets from collections efficiently.

mod dynamic_index;

/// Traits and implementations used for factories to interact with widgets.
pub mod widgets;

/// Implementation of asynchronous factories.
mod r#async;
pub mod positions;

/// Implementation of regular factories.
mod sync;

mod data_guard;
use data_guard::DataGuard;

pub use r#async::{AsyncFactoryComponent, AsyncFactoryVecDeque, AsyncFactoryVecDequeGuard};
pub use sync::{
    CloneableFactoryComponent, FactoryComponent, FactoryVecDeque, FactoryVecDequeGuard,
};

pub use crate::channel::{AsyncFactorySender, FactorySender};
pub use dynamic_index::DynamicIndex;
pub use widgets::traits::*;
