use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A dynamic index that updates automatically when items are shifted inside a factory container.
///
/// For example a [`FactoryVecDeque`](super::FactoryVecDeque) has an [`insert`](super::FactoryVecDequeGuard::insert)
/// method that allows users to insert data at arbitrary positions.
/// If we insert at the front all following widgets will be moved by one which would
/// invalidate their indices.
/// To allow widgets in a factory container to send messages with valid indices
/// this type ensures that the indices is always up to date.
///
/// Never send an index as [`usize`] but always as [`DynamicIndex`]
/// to the update function because messages can be queued up and stale by the time they are handled.
///
/// [`DynamicIndex`] is a smart pointer so cloning will work similar to [`std::rc::Rc`] and will create
/// a pointer to the same data.
///
/// In short: only call [`current_index`](DynamicIndex::current_index) from the update function
/// where you actually need the index as [`usize`].
#[derive(Debug)]
pub struct DynamicIndex {
    inner: Arc<AtomicUsize>,
}

impl PartialEq for DynamicIndex {
    fn eq(&self, other: &Self) -> bool {
        self.current_index().eq(&other.current_index())
    }
}

impl Eq for DynamicIndex {}

impl Clone for DynamicIndex {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl DynamicIndex {
    /// Get the current index number.
    ///
    /// This value is updated by the factory container and might change after each update function.
    #[must_use]
    pub fn current_index(&self) -> usize {
        self.inner.load(Ordering::Relaxed)
    }

    // Creates a [`WeakDynamicIndex`] for sending in messages.
    // pub fn downgrade(&self) -> WeakDynamicIndex {
    //     WeakDynamicIndex {
    //         inner: Arc::downgrade(&self.inner),
    //     }
    // }

    pub(super) fn increment(&self) {
        self.inner.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn decrement(&self) {
        self.inner.fetch_sub(1, Ordering::Relaxed);
    }

    pub(super) fn set_value(&self, new_value: usize) {
        self.inner.store(new_value, Ordering::Relaxed);
    }

    pub fn new(index: usize) -> Self {
        Self {
            inner: Arc::new(AtomicUsize::new(index)),
        }
    }
}

// A weak version of [`DynamicIndex`].
//
// Use this to send messages to the update function and call [`upgrade`](WeakDynamicIndex::upgrade)
// to receive the actual [`DynamicIndex`].
//
// A weak index is preferred for sending in messages because messages can be stale by the time they
// are handled and the element already deleted. A weak reference doesn't keep the index alive
// if the element was deleted which allows you to properly handle invalid indices.
//
// # Panics
//
// Sending a [`WeakDynamicIndex`] to a different thread and accessing it will panic.
// #[derive(Debug)]
// pub struct WeakDynamicIndex {
//     inner: Weak<Mutex<usize>>,
// }

// impl Clone for WeakDynamicIndex {
//     fn clone(&self) -> Self {
//         WeakDynamicIndex {
//             inner: self.inner.clone(),
//         }
//     }
// }

// impl WeakDynamicIndex {
// Attempts to upgrade the [`WeakDynamicIndex`] to a [`DynamicIndex`].
//
// Returns [`None`] if the index has since been dropped.
//     pub fn upgrade(&self) -> Option<DynamicIndex> {
//         Weak::upgrade(&self.inner).map(|inner| {
//             DynamicIndex {
//                 inner,
//             }
//         })
//     }
// }
