//! Containers similar to [`std::collections`] that implement [`Factory`](super::Factory)

mod factory_vec;
mod factory_vec_deque;

pub use factory_vec::FactoryVec;
pub use factory_vec_deque::{DynamicIndex, FactoryVecDeque};
