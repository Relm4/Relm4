use crate::{Receiver, Sender};

use crate::factory::sync::builder::FactoryBuilder;
use crate::factory::sync::handle::FactoryHandle;
use crate::factory::{CloneableFactoryComponent, FactoryComponent, FactoryView};

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops;

#[derive(Debug)]
#[must_use]
pub struct FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    inner: &'a mut FactoryHandle<C>,
}

impl<'a, C> ops::Deref for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.inner.data.get()
    }
}

impl<'a, C> ops::DerefMut for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.data.get_mut()
    }
}

impl<'a, C> Drop for FactoryElementGuard<'a, C>
where
    C: FactoryComponent,
{
    fn drop(&mut self) {
        self.inner.notifier.send(()).unwrap()
    }
}

#[derive(Debug)]
/// A builder-pattern struct for building a [`FactoryHashMap`].
pub struct FactoryHashMapBuilder<K, C: FactoryComponent, S = RandomState> {
    hasher: S,
    _component: PhantomData<C>,
    _key: PhantomData<K>,
}

impl<K, C> Default for FactoryHashMapBuilder<K, C>
where
    C: FactoryComponent,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, C> FactoryHashMapBuilder<K, C>
where
    C: FactoryComponent,
    C::ParentWidget: Default,
{
    #[must_use]
    /// Launch the factory with a default parent widget.
    pub fn launch_default(self) -> FactoryHashMapConnector<K, C> {
        self.launch(Default::default())
    }
}

impl<K, C> FactoryHashMapBuilder<K, C>
where
    C: FactoryComponent,
{
    /// Creates a new [`FactoryHashMapBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            hasher: RandomState::default(),
            _component: PhantomData,
            _key: PhantomData,
        }
    }

    /// Sets a different hasher.
    pub fn hasher<H: Hasher>(self, hasher: H) -> FactoryHashMapBuilder<K, C, H> {
        let Self {
            _component, _key, ..
        } = self;

        FactoryHashMapBuilder {
            hasher,
            _component,
            _key,
        }
    }

    /// Launch the factory.
    /// This is similar to [`Connector::launch`](crate::component::ComponentBuilder::launch).
    pub fn launch(self, widget: C::ParentWidget) -> FactoryHashMapConnector<K, C> {
        let Self { hasher, _key, .. } = self;

        let (output_sender, output_receiver) = crate::channel();

        FactoryHashMapConnector {
            widget,
            output_sender,
            output_receiver,
            hasher,
            _key,
        }
    }
}

#[derive(Debug)]
/// Second stage of the builder-pattern for building a [`FactoryHashMap`].
pub struct FactoryHashMapConnector<K, C, S = RandomState>
where
    C: FactoryComponent,
{
    widget: C::ParentWidget,
    output_sender: Sender<C::Output>,
    output_receiver: Receiver<C::Output>,
    hasher: S,
    _key: PhantomData<K>,
}

impl<K, C> FactoryHashMapConnector<K, C>
where
    C: FactoryComponent,
{
    /// Forwards output events to the designated sender.
    pub fn forward<F, Msg>(self, sender_: &Sender<Msg>, f: F) -> FactoryHashMap<K, C>
    where
        F: Fn(C::Output) -> Msg + Send + 'static,
        C::Output: Send,
        Msg: Send + 'static,
    {
        let Self {
            widget,
            output_sender,
            output_receiver,
            hasher,
            ..
        } = self;

        let sender_clone = sender_.clone();

        crate::spawn(async move {
            while let Some(msg) = output_receiver.recv().await {
                if sender_clone.send(f(msg)).is_err() {
                    break;
                }
            }
        });

        FactoryHashMap {
            widget,
            output_sender,
            inner: HashMap::with_hasher(hasher),
        }
    }

    /// Ignore outputs from the component and finish the builder.
    pub fn detach(self) -> FactoryHashMap<K, C> {
        let Self {
            widget,
            output_sender,
            hasher,
            ..
        } = self;

        FactoryHashMap {
            widget,
            output_sender,
            inner: HashMap::with_hasher(hasher),
        }
    }
}

/// A container similar to [`HashMap`] that can be used to store
/// values of type [`FactoryComponent`].
#[derive(Debug)]
pub struct FactoryHashMap<K, C: FactoryComponent, S = RandomState> {
    widget: C::ParentWidget,
    output_sender: Sender<C::Output>,
    inner: HashMap<K, FactoryHandle<C>, S>,
}

impl<K, C, S> Drop for FactoryHashMap<K, C, S>
where
    C: FactoryComponent,
{
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K, C, S> ops::Index<&K> for FactoryHashMap<K, C, S>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = C;

    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Called `get` on an invalid key")
    }
}

impl<K, C> FactoryHashMap<K, C, RandomState>
where
    C: FactoryComponent,
{
    /// Creates a new [`FactoryHashMap`].
    #[must_use]
    pub fn builder() -> FactoryHashMapBuilder<K, C> {
        FactoryHashMapBuilder::new()
    }
}

impl<K, C, S> FactoryHashMap<K, C, S>
where
    C: FactoryComponent,
{
    /// Returns the number of elements in the [`FactoryHashMap`].
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the [`FactoryHashMap`] is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Send clone of a message to all of the elements.
    pub fn broadcast(&self, msg: C::Input)
    where
        C::Input: Clone,
    {
        self.inner.values().for_each(|c| c.input.emit(msg.clone()));
    }

    /// Returns the widget all components are attached to.
    pub const fn widget(&self) -> &C::ParentWidget {
        &self.widget
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&K, &C)> + FusedIterator {
        self.inner.iter().map(|(k, c)| (k, c.data.get()))
    }

    /// Returns an iterator over the factory components.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &C> + FusedIterator {
        self.inner.values().map(|c| c.data.get())
    }

    /// Returns an iterator over the keys of the hash map.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> + FusedIterator {
        self.inner.keys()
    }

    /// Clears the map, removing all factory components.
    pub fn clear(&mut self) {
        for (_, handle) in self.inner.drain() {
            self.widget.factory_remove(&handle.returned_widget);
        }
    }
}

impl<K, C> FactoryHashMap<K, C, RandomState>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
{
    /// Creates a [`FactoryHashMap`] from a [`Vec`].
    pub fn from_vec(component_vec: Vec<(K, C::Init)>, widget: C::ParentWidget) -> Self {
        let mut output = Self::builder().launch(widget).detach();
        for (key, init) in component_vec {
            output.insert(key, init);
        }
        output
    }
}

impl<K, C, S> FactoryHashMap<K, C, S>
where
    C: FactoryComponent<Index = K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Send a mage to one of the elements.
    pub fn send(&self, key: &K, msg: C::Input) {
        self.inner[key].input.emit(msg);
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `key` is invalid.
    pub fn get(&self, key: &K) -> Option<&C> {
        self.inner.get(key).map(|c| c.data.get())
    }

    /// Tries to get a mutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `key` is invalid.
    pub fn get_mut(&mut self, key: &K) -> Option<FactoryElementGuard<'_, C>> {
        self.inner
            .get_mut(key)
            .map(|c| FactoryElementGuard { inner: c })
    }

    /// Inserts a new factory component into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be == without being identical. See the module-level documentation for more.
    pub fn insert(&mut self, key: K, init: C::Init) -> Option<C> {
        let existing = self.remove(&key);

        let builder = FactoryBuilder::new(&key, init, self.output_sender.clone());

        let position = C::position(&builder.data, &key);
        let returned_widget = self
            .widget
            .factory_append(builder.root_widget.clone(), &position);

        let component = builder.launch(&key, returned_widget);

        assert!(self.inner.insert(key, component).is_none());

        existing
    }

    /// Removes a key from the map, returning the factory component at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<C> {
        if let Some(handle) = self.inner.remove(key) {
            self.widget.factory_remove(&handle.returned_widget);
            Some(handle.data.into_inner())
        } else {
            None
        }
    }
}

/// Implements the Clone Trait for [`FactoryHashMap`] if the component implements [`CloneableFactoryComponent`].
impl<K, C> Clone for FactoryHashMap<K, C, RandomState>
where
    C: CloneableFactoryComponent,
    K: Clone + Hash + Eq,
    C: FactoryComponent<Index = K>,
{
    fn clone(&self) -> Self {
        // Create a new, empty FactoryHashMap.
        let mut clone = FactoryHashMap::builder()
            .launch(self.widget.clone())
            .detach();
        // Iterate over the items in the original FactoryHashMap.
        for (k, item) in self.iter() {
            // Clone each item and push it onto the new FactoryHashMap.
            let init = C::get_init(item);
            clone.insert(k.clone(), init);
        }
        // Return the new, cloned FactoryHashMap.
        clone
    }
}
