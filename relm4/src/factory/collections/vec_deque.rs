use crate::Sender;

use crate::factory::{
    builder::FactoryBuilder, component_storage::ComponentStorage, DynamicIndex, FactoryComponent,
    FactoryView, Position,
};

use gtk::prelude::{Cast, IsA};

use super::{ModelStateValue, RenderedState};

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Index, IndexMut};

#[derive(Debug)]
pub struct FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    inner: &'a mut FactoryVecDeque<Widget, C, ParentMsg>,
}

impl<'a, Widget, C, ParentMsg> Drop for FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    fn drop(&mut self) {
        self.inner.render_changes();
    }
}

impl<'a, Widget, C, ParentMsg> FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    fn new(inner: &'a mut FactoryVecDeque<Widget, C, ParentMsg>) -> Self {
        let mut guard = FactoryVecDequeGuard { inner };

        #[cfg(feature = "libadwaita")]
        guard.apply_external_updates();

        guard
    }

    /// Apply external updates that happened between the last render.
    ///
    /// [`FactoryVecDeque`] should not be edited between calling [`Self::render_changes`]
    /// and this method, as it might cause undefined behaviour. This shouldn't be possible
    /// because [`FactoryVecDequeGuard`] calls this method on creation.
    #[cfg(feature = "libadwaita")]
    fn apply_external_updates(&mut self) {
        if let Some(tab_view) = self.inner.widget().dynamic_cast_ref::<adw::TabView>() {
            let length = tab_view.n_pages();
            let mut hashes: Vec<u64> = Vec::with_capacity(length as usize);

            for i in 0..length {
                let page = tab_view.nth_page(i);
                let mut hasher = DefaultHasher::default();
                page.hash(&mut hasher);
                hashes.push(hasher.finish());
            }

            for (index, hash) in hashes.iter().enumerate() {
                if *hash
                    != self
                        .inner
                        .rendered_state
                        .get(index)
                        .map(|state| state.widget_hash)
                        .unwrap_or_default()
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
        }
    }

    /// Tries to get a mutable reference to
    /// the model of one element.
    ///
    /// Returns `None` is `index` is invalid.
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
    /// Returns None if the deque is empty.
    pub fn back_mut(&mut self) -> Option<&mut C> {
        self.get_mut(self.len().wrapping_sub(1))
    }

    /// Provides a mutable reference to the model of the front element.
    ///
    /// Returns None if the deque is empty.
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
    pub fn push_back(&mut self, init_params: C::InitParams) {
        let index = self.len();
        self.insert(index, init_params);
    }

    /// Prepends an element to the [`FactoryVecDeque`].
    pub fn push_front(&mut self, init_params: C::InitParams) {
        self.insert(0, init_params);
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
    pub fn insert(&mut self, index: usize, init_params: C::InitParams) {
        let dyn_index = DynamicIndex::new(index);

        // Increment the indexes of the following elements.
        for states in self.inner.model_state.iter_mut().skip(index) {
            states.index.increment();
        }

        let builder = FactoryBuilder::new(&dyn_index, init_params);

        self.inner
            .components
            .insert(index, ComponentStorage::Builder(builder));
        self.inner.model_state.insert(
            index,
            ModelStateValue {
                index: dyn_index,
                uid: self.uid_counter,
                changed: false,
            },
        );
        self.inner.uid_counter += 1;
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
        self.move_to(current_position, 0)
    }

    /// Moves an element at index `current_position` to the back,
    /// shifting all elements between these positions.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn move_back(&mut self, current_position: usize) {
        self.move_to(current_position, self.len() - 1)
    }

    /// Remove all components from the [`FactoryVecDeque`].
    pub fn clear(&mut self) {
        self.inner.model_state.clear();

        for component in self.inner.components.drain(..) {
            if let Some(widget) = component.returned_widget() {
                self.inner.widget.factory_remove(widget);
            }
        }
    }
}

impl<'a, Widget, C, ParentMsg> Deref for FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    type Target = FactoryVecDeque<Widget, C, ParentMsg>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, Widget, C, ParentMsg> Index<usize> for FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<'a, Widget, C, ParentMsg> IndexMut<usize> for FactoryVecDequeGuard<'a, Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("Called `get_mut` on an invalid index")
    }
}

/// A container similar to [`VecDeque`] that can be used to store
/// data associated with components that implement [`FactoryComponent`].
#[derive(Debug)]
pub struct FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    widget: Widget,
    parent_sender: Sender<ParentMsg>,
    components: VecDeque<ComponentStorage<Widget, C, ParentMsg>>,
    model_state: VecDeque<ModelStateValue>,
    rendered_state: VecDeque<RenderedState>,
    uid_counter: u16,
}

impl<Widget, C, ParentMsg> Drop for FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    fn drop(&mut self) {
        for component in &mut self.components {
            if let Some(widget) = component.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }
    }
}

impl<Widget, C, ParentMsg> Index<usize> for FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Called `get` on an invalid index")
    }
}

impl<Widget, C, ParentMsg> FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    /// Creates a new [`FactoryVecDeque`].
    pub fn new(widget: Widget, parent_sender: &Sender<ParentMsg>) -> Self {
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

    pub fn guard(&mut self) -> FactoryVecDequeGuard<'_, Widget, C, ParentMsg> {
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
        self.components[index].send(msg)
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns `None` is `index` is invalid.
    pub fn get(&self, index: usize) -> Option<&C> {
        // Safety: This is safe because ownership is tracked by each
        // component individually, an therefore violating ownership
        // rules is impossible.
        // The safe version struggles with lifetime, maybe this can
        // be fixed soon.
        self.components.get(index).map(ComponentStorage::get)
    }

    /// Provides a reference to the model of the back element.
    ///
    /// Returns None if the deque is empty.
    pub fn back(&self) -> Option<&C> {
        self.get(self.len().wrapping_sub(1))
    }

    /// Provides a reference to the model of the front element.
    ///
    /// Returns None if the deque is empty.
    pub fn front(&self) -> Option<&C> {
        self.get(0)
    }

    /// Returns the widget all components are attached to.
    pub fn widget(&self) -> &Widget {
        &self.widget
    }
}
