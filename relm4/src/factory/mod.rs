//! Defines traits and data types to generate widgets from collections efficiently.

mod builder;
mod component_storage;
mod handle;
mod widgets;

pub mod collections;
mod runtime_dropper;
pub use collections::*;

mod dynamic_index;
pub use dynamic_index::DynamicIndex;

pub mod positions;
pub use positions::*;

pub mod traits;
pub use traits::*;
