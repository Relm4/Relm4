use crate::Sender;

use crate::factory::sync::builder::FactoryBuilder;
use crate::factory::sync::component_storage::ComponentStorage;
use crate::factory::sync::traits::CloneableFactoryComponent;
use crate::factory::{DynamicIndex, FactoryComponent, FactoryView};

use super::{ModelStateValue, RenderedState};

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::Hash;
use std::iter::FusedIterator;
use std::ops::{Deref, Index, IndexMut};

#[cfg(feature = "libadwaita")]
use gtk::prelude::Cast;

#[cfg(feature = "libadwaita")]
use std::hash::Hasher;

/// Provides methods to edit the underlying [`FactoryVecDeque`].
///
/// The changes will be rendered on the widgets after the guard goes out of scope.
#[derive(Debug)]
#[must_use]
pub struct FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    inner: &'a mut FactoryVecDeque<C>,
}

impl<'a, C> Drop for FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    fn drop(&mut self) {
        self.inner.render_changes();
    }
}

impl<'a, C> FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    fn new(inner: &'a mut FactoryVecDeque<C>) -> Self {
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return)]
        let mut guard = FactoryVecDequeGuard { inner };

        #[cfg(feature = "libadwaita")]
        guard.apply_external_updates();

        guard
    }

    /// Drops the guard and renders all changes.
    ///
    /// Use this to transfer full ownership back to the [`FactoryVecDeque`].
    pub fn drop(self) {
        drop(self);
    }

    /// Apply external updates that happened between the last render.
    ///
    /// [`FactoryVecDeque`] should not be edited between calling [`Self::render_changes`]
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
    /// Returns [`None`] if `index` is invalid.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut C> {
        // Mark as modified
        if let Some(state) = self.inner.model_state.get_mut(index) {
            state.changed = true;
        }
        self.inner
            .components
            .get_mut(index)
            .map(ComponentStorage::get_mut)
    }

    /// Provides a mutable reference to the model of the back element.
    ///
    ///  Returns [`None`] if the deque is empty.
    pub fn back_mut(&mut self) -> Option<&mut C> {
        self.get_mut(self.len().wrapping_sub(1))
    }

    /// Provides a mutable reference to the model of the front element.
    ///
    ///  Returns [`None`] if the deque is empty.
    pub fn front_mut(&mut self) -> Option<&mut C> {
        self.get_mut(0)
    }

    /// Removes the last element from the [`FactoryVecDeque`] and returns it,
    /// or [`None`] if it is empty.
    pub fn pop_back(&mut self) -> Option<C> {
        if self.is_empty() {
            None
        } else {
            self.remove(self.len() - 1)
        }
    }

    /// Removes the first element from the [`FactoryVecDeque`] and returns it,
    /// or [`None`] if it is empty.
    pub fn pop_front(&mut self) -> Option<C> {
        self.remove(0)
    }

    /// Removes and returns the element at index from the [`FactoryVecDeque`].
    /// Returns [`None`] if index is out of bounds.
    ///
    /// Element at index 0 is the front of the queue.
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

        component.map(ComponentStorage::extract)
    }

    /// Appends an element at the end of the [`FactoryVecDeque`].
    pub fn push_back(&mut self, init: C::Init) -> DynamicIndex {
        let index = self.len();
        self.insert(index, init)
    }

    /// Prepends an element to the [`FactoryVecDeque`].
    pub fn push_front(&mut self, init: C::Init) -> DynamicIndex {
        self.insert(0, init)
    }

    /// Inserts an element at index within the [`FactoryVecDeque`],
    /// shifting all elements with indices greater than or equal
    /// to index towards the back.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics if index is greater than [`FactoryVecDeque`]’s length.
    pub fn insert(&mut self, index: usize, init: C::Init) -> DynamicIndex {
        let dyn_index = DynamicIndex::new(index);

        // Increment the indexes of the following elements.
        for states in self.inner.model_state.iter_mut().skip(index) {
            states.index.increment();
        }

        let builder = FactoryBuilder::new(&dyn_index, init);

        self.inner
            .components
            .insert(index, ComponentStorage::Builder(builder));
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

    /// Remove all components from the [`FactoryVecDeque`].
    pub fn clear(&mut self) {
        self.inner.model_state.clear();

        for component in self.inner.components.drain(..) {
            // Remove all widgets
            if let Some(widget) = component.returned_widget() {
                self.inner.widget.factory_remove(widget);
            }

            // Make sure the component is shutdown properly
            component.extract();
        }

        self.inner.rendered_state.clear();
        self.inner.uid_counter = 1;
    }

    /// Returns an iterator over the components that returns mutable references.
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut C> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
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

impl<'a, C> Deref for FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    type Target = FactoryVecDeque<C>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, C> Index<usize> for FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<'a, C> IndexMut<usize> for FactoryVecDequeGuard<'a, C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("Called `get_mut` on an invalid index")
    }
}

/// A container similar to [`VecDeque`] that can be used to store
/// data associated with components that implement [`FactoryComponent`].
///
/// To access mutable methods of the factory, create a guard using [`Self::guard`].
#[derive(Debug)]
pub struct FactoryVecDeque<C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    widget: C::ParentWidget,
    parent_sender: Sender<C::ParentInput>,
    components: VecDeque<ComponentStorage<C>>,
    model_state: VecDeque<ModelStateValue>,
    rendered_state: VecDeque<RenderedState>,
    uid_counter: usize,
}

impl<C> Drop for FactoryVecDeque<C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    fn drop(&mut self) {
        self.guard().clear();
    }
}

impl<C> Index<usize> for FactoryVecDeque<C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Called `get` on an invalid index")
    }
}

impl<C> FactoryVecDeque<C>
where
    C: FactoryComponent<Index = DynamicIndex>,
{
    /// Creates a new [`FactoryVecDeque`].
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

    /// Provides a [`FactoryVecDequeGuard`] that can be used to edit the factory.
    ///
    /// The changes will be rendered on the widgets after the guard goes out of scope.
    pub fn guard(&mut self) -> FactoryVecDequeGuard<'_, C> {
        FactoryVecDequeGuard::new(self)
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
                let comp = &components[index];
                let insert_widget = comp.widget();
                let position = C::position(comp.get(), &state.index);
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
                let position = C::position(comp.get(), &self.model_state[index].index);
                self.widget
                    .factory_update_position(comp.returned_widget().unwrap(), &position);
            }
        }
    }

    /// Returns the number of elements in the [`FactoryVecDeque`].
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns true if the [`FactoryVecDeque`] is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Send a message to one of the elements.
    pub fn send(&self, index: usize, msg: C::Input) {
        self.components[index].send(msg);
    }

    /// Send clone of a message to all of the elements.
    pub fn broadcast(&self, msg: C::Input)
    where
        C::Input: Clone,
    {
        self.components.iter().for_each(|c| c.send(msg.clone()));
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns [`None`] if `index` is invalid.
    pub fn get(&self, index: usize) -> Option<&C> {
        self.components.get(index).map(ComponentStorage::get)
    }

    /// Provides a reference to the model of the back element.
    ///
    ///  Returns [`None`] if the deque is empty.
    pub fn back(&self) -> Option<&C> {
        self.get(self.len().wrapping_sub(1))
    }

    /// Provides a reference to the model of the front element.
    ///
    ///  Returns [`None`] if the deque is empty.
    pub fn front(&self) -> Option<&C> {
        self.get(0)
    }

    /// Returns the widget all components are attached to.
    pub const fn widget(&self) -> &C::ParentWidget {
        &self.widget
    }

    /// Returns an iterator over the components.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &C> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        self.components.iter().map(ComponentStorage::get)
    }

    /// Creates a FactoryVecDeque from any IntoIterator
    pub fn from_iter(
        component_iter: impl IntoIterator<Item = C::Init>,
        widget: C::ParentWidget,
        parent_sender: &Sender<C::ParentInput>,
    ) -> Self {
        let mut output = Self::new(widget, parent_sender);
        {
            let mut edit = output.guard();
            for component in component_iter {
                edit.push_back(component);
            }
            edit.drop();
        }
        output
    }
}

///Implements the Clone Trait for `FactoryVecDeque<C>` where C is Cloneable
impl<C> Clone for FactoryVecDeque<C>
where
    C: CloneableFactoryComponent + FactoryComponent<Index = DynamicIndex>,
{
    fn clone(&self) -> Self {
        // Create a new, empty FactoryVecDeque.
        let mut clone = FactoryVecDeque::new(self.widget.clone(), &self.parent_sender.clone());
        // Iterate over the items in the original FactoryVecDeque.
        for item in self.iter() {
            // Clone each item and push it onto the new FactoryVecDeque.
            let init = C::get_init(item);
            clone.guard().push_back(init);
        }
        // Return the new, cloned FactoryVecDeque.
        clone
    }
}
