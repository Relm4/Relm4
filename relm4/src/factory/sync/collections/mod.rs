//! Containers similar to [`std::collections`] that can be used to store factory data.

mod hashmap;
mod vec_deque;

pub use hashmap::{FactoryHashMap, FactoryHashMapBuilder, FactoryHashMapConnector};
pub use vec_deque::{
    FactoryVecDeque, FactoryVecDequeBuilder, FactoryVecDequeConnector, FactoryVecDequeGuard,
};

use crate::factory::DynamicIndex;

#[derive(Debug)]
struct RenderedState {
    uid: usize,
    #[cfg(feature = "libadwaita")]
    widget_hash: u64,
}

#[derive(Debug)]
struct ModelStateValue {
    index: DynamicIndex,
    uid: usize,
    changed: bool,
}
