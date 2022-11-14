mod builder;
mod collections;
mod component_storage;
mod data_guard;
mod handle;
mod traits;

use builder::FactoryBuilder;
pub use collections::{FactoryVecDeque, FactoryVecDequeGuard};
use data_guard::DataGuard;
use handle::FactoryHandle;
pub use traits::FactoryComponent;
