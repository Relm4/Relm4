//! Defines traits and data types to generate widgets from collections efficiently.

mod builder;
mod component_storage;
mod data_guard;
mod dynamic_index;
mod handle;
mod widgets;

use data_guard::DataGuard;
use handle::FactoryHandle;

pub mod collections;
pub mod positions;
pub mod traits;

pub use crate::sender::FactoryComponentSender;
pub use collections::*;
pub use dynamic_index::DynamicIndex;
pub use positions::*;
pub use traits::*;
