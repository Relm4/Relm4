//! Containers similar to [`std::collections`] that can be used to store factory data.

mod vec_deque;
pub use vec_deque::FactoryVecDeque;

use super::DynamicIndex;

#[derive(Debug)]
struct RenderedState {
    uid: u16,
    widget_hash: u64,
}

#[derive(Debug)]
struct ModelStateValue {
    index: DynamicIndex,
    uid: u16,
    changed: bool,
}
