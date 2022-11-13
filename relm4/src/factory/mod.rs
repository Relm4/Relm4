//! Defines traits and data types to generate widgets from collections efficiently.

mod dynamic_index;

/// Traits and implementations used for factories to interact with widgets.
pub mod widgets;

/// Implementation of asynchronous factories.
pub mod r#async;
pub mod positions;

/// Implementation of regular factories.
pub mod sync;

pub use crate::sender::{AsyncFactoryComponentSender, FactoryComponentSender};
pub use dynamic_index::DynamicIndex;
pub use positions::*;
pub use sync::collections::*;

pub use sync::traits::*;
pub use widgets::traits;
pub use widgets::traits::*;
