use std::{
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use once_cell::sync::Lazy;

use crate::Sender;

use super::SubscriberFn;

/// A type that allows you to share information across your
/// application easily.
/// Get immutable and mutable access to the data and subscribe to changes.
///
/// # Panics
///
/// [`SharedState`] uses a [`RwLock`] internally.
/// If you use [`Self::read()`] and [`Self::write()`] in the same scope
/// your code might be stuck in a deadlock or panic.
pub struct SharedState<Data> {
    data: Lazy<RwLock<Data>>,
    subscribers: Lazy<RwLock<Vec<SubscriberFn<Data>>>>,
}

impl<Data: std::fmt::Debug> std::fmt::Debug for SharedState<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState")
            .field("data", &self.data)
            .field("subscribers", &self.subscribers.try_read().map(|s| s.len()))
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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: Lazy::new(RwLock::default),
            subscribers: Lazy::new(RwLock::default),
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
    ///     let mut data = STATE.write();
    ///     *data += 1;
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
                sender.send(msg).is_ok()
            }));
    }

    /// An alternative version of [`subscribe()`](Self::subscribe()) that only send a message if
    /// the closure returns [`Some`].
    pub fn subscribe_optional<Msg, F>(&self, sender: &Sender<Msg>, f: F)
    where
        F: Fn(&Data) -> Option<Msg> + 'static + Send + Sync,
        Msg: Send + 'static,
    {
        let sender = sender.clone();
        self.subscribers
            .write()
            .unwrap()
            .push(Box::new(move |data: &Data| {
                if let Some(msg) = f(data) {
                    sender.send(msg).is_ok()
                } else {
                    true
                }
            }));
    }

    /// Get immutable access to the shared data.
    ///
    /// Returns an RAII guard which will release this thread’s shared access
    /// once it is dropped.
    ///
    /// The calling thread will be blocked until there are no more writers
    /// which hold the lock (see [`RwLock`]).
    ///
    /// # Panics
    ///
    /// This function will panic if the internal [`RwLock`] is poisoned.
    /// An [`RwLock`] is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    ///
    /// Also, this function might panic when called if the lock is already
    /// held by the current thread.
    pub fn read(&self) -> SharedStateReadGuard<'_, Data> {
        SharedStateReadGuard {
            inner: self.data.read().unwrap(),
        }
    }

    /// Get mutable access to the shared data.
    ///
    /// Returns an RAII guard which will **notify all subscribers** and
    /// release this thread’s shared access once it is dropped.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the internal lock (see [`RwLock`]).
    ///
    /// # Panics
    ///
    /// This function will panic if the internal [`RwLock`] is poisoned.
    /// An [`RwLock`] is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    ///
    /// Also, this function might panic when called if the lock is already
    /// held by the current thread.
    ///
    /// # Example
    ///
    /// ```
    /// # use relm4::SharedState;
    /// static STATE: SharedState<u8> = SharedState::new();
    ///
    /// // Overwrite the current value with 1
    /// *STATE.write() = 1;
    /// ```
    ///
    /// # Panic example
    ///
    /// ```no_run
    /// # use relm4::SharedState;
    /// static STATE: SharedState<u8> = SharedState::new();
    ///
    /// let read_guard = STATE.read();
    ///
    /// // This is fine
    /// let another_read_guard = STATE.read();
    ///
    /// // This might panic or result in a dead lock
    /// // because you cannot read and write at the same time.
    /// // To solve this, drop all read guards on this thread first.
    /// let another_write_guard = STATE.write();
    /// ```
    pub fn write(&self) -> SharedStateWriteGuard<'_, Data> {
        let subscribers = self.subscribers.write().unwrap();
        let data = self.data.write().unwrap();

        SharedStateWriteGuard { data, subscribers }
    }

    /// Get mutable access to the shared data.
    /// Since this call borrows the [`SharedState`] mutably,
    /// no actual locking needs to take place, but the mutable
    /// borrow statically guarantees no locks exist.
    ///
    /// **This method will not notify any subscribers!**
    ///
    /// # Panics
    ///
    /// This function will panic if the internal [`RwLock`] is poisoned.
    /// An [`RwLock`] is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    pub fn get_mut(&mut self) -> &mut Data {
        self.data.get_mut().unwrap()
    }

    /// Get immutable access to the shared data.
    ///
    /// **This method will not notify any subscribers!**
    ///
    /// # Panics
    ///
    /// This function will panic if the internal [`RwLock`] is poisoned.
    /// An [`RwLock`] is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    ///
    /// Also, this function might panic when called if the lock is already
    /// held by the current thread.
    pub fn read_inner(&self) -> RwLockReadGuard<'_, Data> {
        self.data.read().unwrap()
    }

    /// Get mutable access to the shared data.
    ///
    /// **This method will not notify any subscribers!**
    ///
    /// # Panics
    ///
    /// This function will panic if the internal [`RwLock`] is poisoned.
    /// An [`RwLock`] is poisoned whenever a writer panics while holding an exclusive lock.
    /// The failure will occur immediately after the lock has been acquired.
    ///
    /// Also, this function might panic when called if the lock is already
    /// held by the current thread.
    pub fn write_inner(&self) -> RwLockWriteGuard<'_, Data> {
        self.data.write().unwrap()
    }
}

#[derive(Debug)]
/// A guard that immutably dereferences `Data`.
pub struct SharedStateReadGuard<'a, Data> {
    inner: RwLockReadGuard<'a, Data>,
}

impl<'a, Data> Deref for SharedStateReadGuard<'a, Data> {
    type Target = Data;

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
    type Target = Data;

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
        let data = &*self.data;
        // Remove all elements which had their senders dropped.
        self.subscribers.retain(|subscriber| subscriber(data));
    }
}

#[cfg(test)]
mod test {
    use super::SharedState;

    static STATE: SharedState<u8> = SharedState::new();

    #[test]
    fn shared_state() {
        assert_eq!(*STATE.read(), 0);

        {
            let mut data = STATE.write();
            *data += 1;
        }

        assert_eq!(*STATE.read(), 1);

        let (sender, receiver) = crate::channel();

        STATE.subscribe(&sender, |data| *data);

        {
            let mut data = STATE.write();
            *data += 1;
        }

        assert_eq!(receiver.recv_sync().unwrap(), 2);
        assert_eq!(*STATE.read(), 2);
    }
}
