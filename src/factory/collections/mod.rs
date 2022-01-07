//! Containers similar to [`std::collections`] that implement the [`Factory`](super::Factory) trait.
//!
//! # Which factory type to use
//!
//! Use [`FactoryVec`] if you only need to push and pop at the back.
//! If you need more flexibility for example for pushing or removing items at
//! arbitrary indices use [`FactoryVecDeque`].
//!
//! Also, [`FactoryVec`] works with all container widgets that implement
//! [`FactoryView`](super::FactoryView) such as [`gtk::Box`] or [`gtk::Grid`].
//!
//! [`FactoryVecDeque`] additionally needs container widgets to implement the
//! [`FactoryListView`](super::FactoryListView) trait that's implements support
//! for adding and removing widgets at arbitrary positions.
//! [`gtk::Grid`] for example only works with [`FactoryVec`] but not with
//! [`FactoryVecDeque`] because widgets can't be inserted at arbitrary positions.
//!
//! Another difference is that [`FactoryVecDeque`] will insert widgets
//! at the beginning if the container used for the factory contains other widgets
//! that were inserted independently from the factory.
//! Yet, [`FactoryVec`] will insert widgets at the end in the same scenario.

mod factory_vec;
mod factory_vec_deque;

pub use factory_vec::FactoryVec;
pub use factory_vec_deque::{DynamicIndex, FactoryVecDeque, WeakDynamicIndex};

use std::fmt::Debug;

struct Widgets<Widgets: Debug, Root: Debug> {
    widgets: Widgets,
    root: Root,
}

impl<WidgetsType: Debug, Root: Debug> Debug for Widgets<WidgetsType, Root> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widgets")
            .field("widgets", &self.widgets)
            .field("root", &self.root)
            .finish()
    }
}
