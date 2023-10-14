mod builder;
mod collections;
mod component_storage;
mod handle;
mod traits;

use builder::FactoryBuilder;
use handle::FactoryHandle;

pub use collections::{
    FactoryHashMap, FactoryHashMapBuilder, FactoryHashMapConnector, FactoryVecDeque,
    FactoryVecDequeBuilder, FactoryVecDequeConnector, FactoryVecDequeGuard,
};
pub use traits::{CloneableFactoryComponent, FactoryComponent};
