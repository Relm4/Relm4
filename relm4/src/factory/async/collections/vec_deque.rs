use crate::Sender;

use crate::factory::r#async::component_storage::AsyncComponentStorage;
use crate::factory::r#async::traits::AsyncFactoryComponent;
use crate::factory::r#async::AsyncFactoryBuilder;
use crate::factory::{DynamicIndex, FactoryView};

use super::{ModelStateValue, RenderedState};

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::Hash;
use std::iter::FusedIterator;
use std::ops::Deref;

#[cfg(feature = "libadwaita")]
use gtk::prelude::Cast;

#[cfg(feature = "libadwaita")]
use std::hash::Hasher;

/// Provides methods to edit the underlying [`AsyncFactoryVecDeque`].
///
/// The changes will be rendered on the widgets after the guard goes out of scope.
#[derive(Debug)]
#[must_use]
pub struct AsyncFactoryVecDequeGuard<'a, C: AsyncFactoryComponent>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    inner: &'a mut AsyncFactoryVecDeque<C>,
}

impl<'a, C: AsyncFactoryComponent> Drop for AsyncFactoryVecDequeGuard<'a, C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    fn drop(&mut self) {
        self.inner.render_changes();
    }
}

impl<'a, C: AsyncFactoryComponent> AsyncFactoryVecDequeGuard<'a, C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    fn new(inner: &'a mut AsyncFactoryVecDeque<C>) -> Self {
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return)]
        let mut guard = AsyncFactoryVecDequeGuard { inner };

        #[cfg(feature = "libadwaita")]
        guard.apply_external_updates();

        guard
    }

    /// Drops the guard and renders all changes.
    ///
    /// Use this to transfer full ownership back to the [`AsyncFactoryVecDeque`].
    pub fn drop(self) {
        drop(self);
    }

    /// Apply external updates that happened between the last render.
    ///
    /// [`AsyncFactoryVecDeque`] should not be edited between calling [`Self::render_changes`]
    /// and this method, as it might cause undefined behaviour. This shouldn't be possible
    /// because the method is called in [`FactoryVecDequeGuard::new`].
    #[cfg(feature = "libadwaita")]
    fn apply_external_updates(&mut self) {
        if let Some(tab_view) = self.inner.widget().dynamic_cast_ref::<adw::TabView>() {
            let length = tab_view.n_pages();
            let mut hash_values: Vec<u64> = Vec::with_capacity(usize::try_from(length).unwrap());

            for i in 0..length {
                let page = tab_view.nth_page(i);
                let mut hasher = DefaultHasher::default();
                page.hash(&mut hasher);
                hash_values.push(hasher.finish());
            }

            // Tab rearrangement
            for (index, hash) in hash_values.iter().enumerate() {
                if self
                    .inner
                    .rendered_state
                    .get(index)
                    .map(|state| state.widget_hash)
                    == Some(*hash)
                {
                    let old_position = self
                        .inner
                        .rendered_state
                        .iter()
                        .position(|state| state.widget_hash == *hash)
                        .expect("A new widget was added");

                    let elem = self.inner.rendered_state.remove(old_position).unwrap();
                    self.inner.rendered_state.insert(index, elem);

                    self.move_to(old_position, index);
                }
            }

            // Closed tabs
            let mut index = 0;
            while index < self.inner.rendered_state.len() {
                let hash = self.inner.rendered_state[index].widget_hash;
                if hash_values.contains(&hash) {
                    index += 1;
                } else {
                    self.inner.rendered_state.remove(index);

                    self.remove(index);
                }
            }
        }
    }

    /// Tries to get a mutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `index` is invalid or the async [`init_model()`] method
    /// hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn get_mut(&mut self, index: usize) -> Option<&mut C> {
        // Mark as modified
        if let Some(state) = self.inner.model_state.get_mut(index) {
            state.changed = true;
        }
        self.inner
            .components
            .get_mut(index)
            .and_then(AsyncComponentStorage::get_mut)
    }

    /// Provides a mutable reference to the model of the back element.
    ///
    ///  Returns [`None`] if the deque is empty or the async [`init_model()`] method
    /// of the last element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn back_mut(&mut self) -> Option<&mut C> {
        self.get_mut(self.len().wrapping_sub(1))
    }

    /// Provides a mutable reference to the model of the front element.
    ///
    ///  Returns [`None`] if the deque is empty or the async [`init_model()`] method
    /// of the first element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn front_mut(&mut self) -> Option<&mut C> {
        self.get_mut(0)
    }

    /// Removes the last element from the [`AsyncFactoryVecDeque`] and returns it,
    /// or [`None`] if it is empty or the async [`init_model()`] method
    /// of the element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn pop_back(&mut self) -> Option<C> {
        if self.is_empty() {
            None
        } else {
            self.remove(self.len() - 1)
        }
    }

    /// Removes the first element from the [`AsyncFactoryVecDeque`] and returns it,
    /// or [`None`] if it is empty or the async [`init_model()`] method
    /// of the element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn pop_front(&mut self) -> Option<C> {
        self.remove(0)
    }

    /// Removes and returns the element at index from the [`AsyncFactoryVecDeque`].
    /// or [`None`] if it is empty or the async [`init_model()`] method
    /// of the element hasn't returned yet.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn remove(&mut self, index: usize) -> Option<C> {
        self.inner.model_state.remove(index);
        let component = self.inner.components.remove(index);

        // Decrement the indexes of the following elements.
        for states in self.inner.model_state.iter_mut().skip(index) {
            states.index.decrement();
        }

        if let Some(comp) = &component {
            if let Some(widget) = &comp.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }

        component.and_then(AsyncComponentStorage::extract)
    }

    /// Appends an element at the end of the [`AsyncFactoryVecDeque`].
    pub fn push_back(&mut self, init: C::Init) -> DynamicIndex {
        let index = self.len();
        self.insert(index, init)
    }

    /// Prepends an element to the [`AsyncFactoryVecDeque`].
    pub fn push_front(&mut self, init: C::Init) -> DynamicIndex {
        self.insert(0, init)
    }

    /// Inserts an element at index within the [`AsyncFactoryVecDeque`],
    /// shifting all elements with indices greater than or equal
    /// to index towards the back.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if index is greater than [`AsyncFactoryVecDeque`]’s length.
    pub fn insert(&mut self, index: usize, init: C::Init) -> DynamicIndex {
        let dyn_index = DynamicIndex::new(index);

        // Increment the indexes of the following elements.
        for states in self.inner.model_state.iter_mut().skip(index) {
            states.index.increment();
        }

        let builder = AsyncFactoryBuilder::new(init);

        self.inner
            .components
            .insert(index, AsyncComponentStorage::Builder(builder));
        self.inner.model_state.insert(
            index,
            ModelStateValue {
                index: dyn_index.clone(),
                uid: self.uid_counter,
                changed: false,
            },
        );
        self.inner.uid_counter += 1;

        dyn_index
    }

    /// Swaps elements at indices `first` and `second`.
    ///
    /// `first` and `second` may be equal.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    pub fn swap(&mut self, first: usize, second: usize) {
        // Don't update anything if both are equal
        if first != second {
            self.inner.model_state.swap(first, second);
            self.inner.components.swap(first, second);

            // Update indexes.
            self.model_state[first].index.set_value(first);
            self.model_state[second].index.set_value(second);
        }
    }

    /// Moves an element at index `current_position` to `target`,
    /// shifting all elements between these positions.
    ///
    /// `current_position` and `target` may be equal.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    pub fn move_to(&mut self, current_position: usize, target: usize) {
        // Don't update anything if both are equal
        if current_position != target {
            let elem = self.inner.model_state.remove(current_position).unwrap();
            // Set new index
            elem.index.set_value(target);
            self.inner.model_state.insert(target, elem);

            let comp = self.inner.components.remove(current_position).unwrap();
            self.inner.components.insert(target, comp);

            // Update indexes.
            if current_position > target {
                // Move down -> shift elements in between up.
                for state in self
                    .inner
                    .model_state
                    .iter_mut()
                    .skip(target + 1)
                    .take(current_position - target)
                {
                    state.index.increment();
                }
            } else {
                // Move up -> shift elements in between down.
                for state in self
                    .inner
                    .model_state
                    .iter_mut()
                    .skip(current_position)
                    .take(target - current_position)
                {
                    state.index.decrement();
                }
            }
        }
    }

    /// Moves an element at index `current_position` to the front,
    /// shifting all elements between these positions.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn move_front(&mut self, current_position: usize) {
        self.move_to(current_position, 0);
    }

    /// Moves an element at index `current_position` to the back,
    /// shifting all elements between these positions.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn move_back(&mut self, current_position: usize) {
        self.move_to(current_position, self.len() - 1);
    }

    /// Remove all components from the [`AsyncFactoryVecDeque`].
    pub fn clear(&mut self) {
        self.inner.model_state.clear();

        for component in self.inner.components.drain(..) {
            if let Some(widget) = component.returned_widget() {
                self.inner.widget.factory_remove(widget);
            }
        }
    }

    /// Returns an iterator over the components that returns mutable references.
    ///
    /// Each item will be [`Some`] if the async [`init_model()`] method
    /// of the item returned and otherwise [`None`].
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = Option<&mut C>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.inner
            .components
            .iter_mut()
            .zip(self.inner.model_state.iter_mut())
            .map(|(component, state)| {
                state.changed = true;
                component.get_mut()
            })
    }
}

impl<'a, C: AsyncFactoryComponent> Deref for AsyncFactoryVecDequeGuard<'a, C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    type Target = AsyncFactoryVecDeque<C>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

/// A container similar to [`VecDeque`] that can be used to store
/// data associated with components that implement [`AsyncFactoryComponent`].
///
/// To access mutable methods of the factory, create a guard using [`Self::guard`].
#[derive(Debug)]
pub struct AsyncFactoryVecDeque<C: AsyncFactoryComponent>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    widget: C::ParentWidget,
    parent_sender: Sender<C::ParentInput>,
    components: VecDeque<AsyncComponentStorage<C>>,
    model_state: VecDeque<ModelStateValue>,
    rendered_state: VecDeque<RenderedState>,
    uid_counter: u16,
}

impl<C: AsyncFactoryComponent> Drop for AsyncFactoryVecDeque<C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    fn drop(&mut self) {
        for component in &mut self.components {
            if let Some(widget) = component.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }
    }
}

impl<C: AsyncFactoryComponent> AsyncFactoryVecDeque<C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    /// Creates a new [`AsyncFactoryVecDeque`].
    #[must_use]
    pub fn new(widget: C::ParentWidget, parent_sender: &Sender<C::ParentInput>) -> Self {
        Self {
            widget,
            parent_sender: parent_sender.clone(),
            components: VecDeque::new(),
            model_state: VecDeque::new(),
            rendered_state: VecDeque::new(),
            // 0 is always an invalid uid
            uid_counter: 1,
        }
    }

    /// Provides a [`AsyncFactoryVecDequeGuard`] that can be used to edit the factory.
    ///
    /// The changes will be rendered on the widgets after the guard goes out of scope.
    pub fn guard(&mut self) -> AsyncFactoryVecDequeGuard<'_, C> {
        AsyncFactoryVecDequeGuard::new(self)
    }

    /// Updates the widgets according to the changes made to the factory.
    /// All updates accumulate until this method is called and are handled
    /// efficiently.
    ///
    /// For example, swapping two elements twice will only swap the data twice,
    /// but won't cause any UI updates.
    ///
    /// Also, only modified elements will be updated.
    fn render_changes(&mut self) {
        let mut first_position_change_idx = None;

        let components = &mut self.components;
        let rendered_state = &mut self.rendered_state;
        for (index, state) in self.model_state.iter().enumerate() {
            if state.uid == rendered_state.front().map(|r| r.uid).unwrap_or_default() {
                // Remove item from previously rendered list
                rendered_state.pop_front();

                if state.changed {
                    // Update component
                    components[index].state_change_notify();
                }
            } else if let Some(rendered_index) =
                rendered_state.iter().position(|r| r.uid == state.uid)
            {
                if first_position_change_idx.is_none() {
                    first_position_change_idx = Some(index);
                }

                // Remove item from previously rendered list
                rendered_state.remove(rendered_index);

                // Detach and re-attach item
                let widget = components[index].returned_widget().unwrap();
                if index == 0 {
                    self.widget.factory_move_start(widget);
                } else {
                    let previous_widget = components[index - 1].returned_widget().unwrap();
                    self.widget.factory_move_after(widget, previous_widget);
                }

                if state.changed {
                    // Update component
                    components[index].state_change_notify();
                }
            } else {
                if first_position_change_idx.is_none() {
                    first_position_change_idx = Some(index);
                }

                // The element doesn't exist yet
                let insert_widget = components[index].widget();
                let position = C::position(index);
                let returned_widget = if index == 0 {
                    self.widget.factory_prepend(insert_widget, &position)
                } else {
                    let previous_widget = components[index - 1].returned_widget().unwrap();
                    self.widget
                        .factory_insert_after(insert_widget, &position, previous_widget)
                };
                let component = components.remove(index).unwrap();
                let dyn_index = &self.model_state[index].index;
                let component = component
                    .launch(dyn_index, returned_widget, &self.parent_sender)
                    .unwrap();
                components.insert(index, component);
            }
        }

        // Reset change tracker
        self.model_state.iter_mut().for_each(|s| s.changed = false);

        // Set rendered state to the state of the model
        // because everything should be up-to-date now.
        self.rendered_state = self
            .model_state
            .iter()
            .zip(components.iter())
            .map(|(s, c)| {
                let mut hasher = DefaultHasher::default();
                c.returned_widget().unwrap().hash(&mut hasher);

                RenderedState {
                    uid: s.uid,
                    #[cfg(feature = "libadwaita")]
                    widget_hash: hasher.finish(),
                }
            })
            .collect();

        if let Some(change_index) = first_position_change_idx {
            for (index, comp) in components.iter().enumerate().skip(change_index) {
                let position = C::position(index);
                self.widget
                    .factory_update_position(comp.returned_widget().unwrap(), &position);
            }
        }
    }

    /// Returns the number of elements in the [`AsyncFactoryVecDeque`].
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns true if the [`AsyncFactoryVecDeque`] is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Send a message to one of the elements.
    pub fn send(&self, index: usize, msg: C::Input) {
        self.components[index].send(msg);
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `index` is invalid or the async [`init_model()`] method
    /// hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn get(&self, index: usize) -> Option<&C> {
        self.components
            .get(index)
            .and_then(AsyncComponentStorage::get)
    }

    /// Provides a reference to the model of the back element.
    ///
    /// Returns [`None`] if `index` is invalid or the async [`init_model()`] method
    /// of the last element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn back(&self) -> Option<&C> {
        self.get(self.len().wrapping_sub(1))
    }

    /// Provides a reference to the model of the front element.
    ///
    /// Returns [`None`] if `index` is invalid or the async [`init_model()`] method
    /// of the first element hasn't returned yet.
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn front(&self) -> Option<&C> {
        self.get(0)
    }

    /// Returns the widget all components are attached to.
    pub const fn widget(&self) -> &C::ParentWidget {
        &self.widget
    }

    /// Returns an iterator over the components.
    ///
    /// Each item will be [`Some`] if the async [`init_model()`] method
    /// of the item returned and otherwise [`None`].
    ///
    /// [`init_model()`]: AsyncFactoryComponent::init_model
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = Option<&C>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.components.iter().map(AsyncComponentStorage::get)
    }
}
