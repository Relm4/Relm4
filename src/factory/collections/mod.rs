//! Containers similar to [`std::collections`] that implement the [`Factory`](super::Factory) trait.
//!
//! # Which factory type to use
//!
//! Use [`FactoryVec`] if you only need to push and pop at the back.
//! If you need more flexibility for example for pushing or removing items at
//! arbitraty indices use [`FactoryVecDeque`].

mod factory_vec;
mod factory_vec_deque;

pub use factory_vec::FactoryVec;
pub use factory_vec_deque::{DynamicIndex, FactoryVecDeque};
