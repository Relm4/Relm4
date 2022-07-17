use std::{
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use once_cell::sync::Lazy;

use crate::Sender;

type SubscriberFn<Data> = Box<dyn Fn(&Data) + 'static + Send + Sync>;

/// A type that allows you to share information across your
/// application easily.
/// Get immutable and mutable access to the data and subscribe to changes.
///
/// # Locking
///
/// [`SharedState`] uses a [`RwLock`] internally.
/// If you use [`Self::get()`] and [`Self::get_mut()`] in the same scope
/// this might cause a panic or a deadlock.
pub struct SharedState<Data> {
    data: Lazy<RwLock<Data>>,
    subscribers: Lazy<RwLock<Vec<SubscriberFn<Data>>>>,
}

impl<Data: std::fmt::Debug> std::fmt::Debug for SharedState<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState")
            .field("data", &self.data)
            .field("subscribers", &self.subscribers.read().map(|s| s.len()))
            .finish()
    }
}

impl<Data> SharedState<Data>
where
    Data: Default,
{
    /// Create a new [`SharedState`] variable.
    ///
    /// The data will be initialized lazily on the first access.
    ///
    /// # Example
    ///
    /// ```
    /// # #[derive(Default)]
    /// # struct MyData;
    /// use relm4::SharedState;
    /// static STATE: SharedState<MyData> = SharedState::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            data: Lazy::new(|| RwLock::new(Data::default())),
            subscribers: Lazy::new(|| RwLock::new(Vec::default())),
        }
    }

    /// Subscribe to a shared state type.
    /// Any subscriber will be notified with a message every time
    /// you modify the shared state using [`Self::get_mut()`].
    ///
    /// ```
    /// use relm4::SharedState;
    /// static STATE: SharedState<u8> = SharedState::new();
    ///
    /// let (sender, receiver) = relm4::channel();
    ///
    /// // Every time we modify the data, we will receive
    /// // the updated value as a message.
    /// STATE.subscribe(&sender, |data| *data);
    ///
    /// {
    ///     let mut data = STATE.get_mut();
    ///     **data += 1;
    /// }
    ///
    /// assert_eq!(receiver.recv_sync().unwrap(), 1);
    /// ```
    pub fn subscribe<Msg, F>(&self, sender: &Sender<Msg>, f: F)
    where
        F: Fn(&Data) -> Msg + 'static + Send + Sync,
        Msg: Send + 'static,
    {
        let sender = sender.clone();
        self.subscribers
            .write()
            .unwrap()
            .push(Box::new(move |data: &Data| {
                let msg = f(data);
                sender.send(msg);
            }));
    }

    /// Get immutable access to the shared data.
    pub fn get(&self) -> SharedStateReadGuard<'_, Data> {
        SharedStateReadGuard {
            inner: self.data.read().unwrap(),
        }
    }

    /// Get mutable access to the shared data.
    /// Once the lock is dropped all subscribers will be notified.
    pub fn get_mut(&self) -> SharedStateWriteGuard<'_, Data> {
        SharedStateWriteGuard {
            data: self.data.write().unwrap(),
            subscribers: self.subscribers.write().unwrap(),
        }
    }

    /// Get mutable access to the shared data.
    /// **This method will not notify any subscribers!**
    pub fn get_mut_raw(&self) -> RwLockWriteGuard<'_, Data> {
        self.data.write().unwrap()
    }
}

#[derive(Debug)]
/// A guard that immutably dereferences `Data`.
pub struct SharedStateReadGuard<'a, Data> {
    inner: RwLockReadGuard<'a, Data>,
}

impl<'a, Data> Deref for SharedStateReadGuard<'a, Data> {
    type Target = RwLockReadGuard<'a, Data>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A guard that mutably dereferences `Data`.
/// Once dropped all subscribers of the [`SharedState`] will be notified.
pub struct SharedStateWriteGuard<'a, Data> {
    data: RwLockWriteGuard<'a, Data>,
    subscribers: RwLockWriteGuard<'a, Vec<SubscriberFn<Data>>>,
}

impl<'a, Data: std::fmt::Debug> std::fmt::Debug for SharedStateWriteGuard<'a, Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedStateWriteGuard")
            .field("data", &self.data)
            .field("subscribers", &self.subscribers.len())
            .finish()
    }
}

impl<'a, Data> Deref for SharedStateWriteGuard<'a, Data> {
    type Target = RwLockWriteGuard<'a, Data>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, Data> DerefMut for SharedStateWriteGuard<'a, Data> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, Data> Drop for SharedStateWriteGuard<'a, Data> {
    // Notify subscribers
    fn drop(&mut self) {
        let data = self.data.deref();
        self.subscribers
            .iter()
            .for_each(|subscriber| subscriber(data));
    }
}

#[cfg(test)]
mod test {
    use super::SharedState;

    static STATE: SharedState<u8> = SharedState::new();

    #[test]
    fn shared_state() {
        assert_eq!(**STATE.get(), 0);

        {
            let mut data = STATE.get_mut();
            **data += 1;
        }

        assert_eq!(**STATE.get(), 1);

        let (sender, receiver) = crate::channel();

        STATE.subscribe(&sender, |data| *data);

        {
            let mut data = STATE.get_mut();
            **data += 1;
        }

        assert_eq!(receiver.recv_sync().unwrap(), 2);
        assert_eq!(**STATE.get(), 2);
    }
}
