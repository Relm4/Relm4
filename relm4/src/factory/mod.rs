//! Defines traits and data types to generate widgets from collections efficiently.

mod builder;
mod component_storage;
mod handle;
mod widgets;

pub mod collections;
mod data_guard;
use std::sync::Arc;

pub use collections::*;

mod dynamic_index;
pub use dynamic_index::DynamicIndex;

pub mod positions;
pub use positions::*;

pub mod traits;
pub use traits::*;

use crate::component::ComponentSenderInner;

/// Contain senders used by the factory component.
pub type FactoryComponentSender<ParentWidget, ParentMsg, C> = Arc<
    ComponentSenderInner<
        <C as FactoryComponent<ParentWidget, ParentMsg>>::Input,
        <C as FactoryComponent<ParentWidget, ParentMsg>>::Output,
        <C as FactoryComponent<ParentWidget, ParentMsg>>::CommandOutput,
    >,
>;
