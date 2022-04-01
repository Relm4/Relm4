//! Defines traits and data types to generate widgets from collections efficiently.

mod builder;
mod component_storage;
mod dynamic_index;
mod handle;
pub mod positions;
pub mod traits;
mod widgets;

use crate::Sender;

pub use dynamic_index::DynamicIndex;
pub use traits::*;

use builder::FactoryBuilder;
use component_storage::ComponentStorage;
use handle::FactoryHandle;

use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

#[allow(missing_debug_implementations)]
/// A container similar to [`VecDeque`] that can be used to store
/// data associated with components that implement [`FactoryComponent`].
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
    components: RefCell<VecDeque<ComponentStorage<Widget, C, ParentMsg>>>,
    model_state: RefCell<VecDeque<ModelStateValue>>,
    rendered_state: RefCell<VecDeque<RenderedState>>,
    uid_counter: u16,
}

struct RenderedState {
    uid: u16,
    widget_hash: u64,
}

struct ModelStateValue {
    index: DynamicIndex,
    uid: u16,
    changed: bool,
}

impl<Widget, C, ParentMsg> Drop for FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg> + Position<Widget::Position>,
    C::Root: AsRef<Widget::Children>,
{
    fn drop(&mut self) {
        for component in self.components.get_mut() {
            if let Some(widget) = component.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }
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
            components: RefCell::new(VecDeque::new()),
            model_state: RefCell::new(VecDeque::new()),
            rendered_state: RefCell::new(VecDeque::new()),
            // 0 is always an invalid uid
            uid_counter: 1,
        }
    }

    /// Updates the widgets according to the changes made to the factory.
    /// All updates accumulate until this method is called and are handled
    /// efficiently.
    ///
    /// For example, swapping two elements twice will only swap the data twice,
    /// but won't cause any UI updates.
    ///
    /// Also, only modified elements will be updated.
    pub fn render_changes(&self) {
        let mut first_position_change_idx = None;

        let mut components = self.components.borrow_mut();
        let mut rendered_state = self.rendered_state.borrow_mut();
        for (index, state) in self.model_state.borrow().iter().enumerate() {
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
                let dyn_index = &self.model_state.borrow()[index].index;
                let component = component
                    .launch(dyn_index, returned_widget, &self.parent_sender)
                    .unwrap();
                components.insert(index, component);
            }
        }

        // Reset change tracker
        self.model_state
            .borrow_mut()
            .iter_mut()
            .for_each(|s| s.changed = false);

        // Set rendered state to the state of the model
        // because everything should be up-to-date now.
        drop(rendered_state);
        self.rendered_state.replace(
            self.model_state
                .borrow()
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
                .collect(),
        );

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
        self.components.borrow().len()
    }

    /// Returns true if the [`FactoryVecDeque`] is empty.
    pub fn is_empty(&self) -> bool {
        self.components.borrow().is_empty()
    }

    /// Send a message to one of the elements.
    pub fn send(&self, index: usize, msg: C::Input) {
        self.components.borrow()[index].send(msg)
    }

    /// Tries to get an immutable reference to
    /// the model of one element.
    ///
    /// Returns `None` is `index` is invalid.
    ///
    /// # Panics
    ///
    /// Panics when the same element was borrowed mutably
    /// somewhere else.
    pub fn try_get(&self, index: usize) -> Option<Ref<'_, C>> {
        // Safety: This is safe because ownership is tracked by each
        // component individually, an therefore violating ownership
        // rules is impossible.
        // The safe version struggles with lifetime, maybe this can
        // be fixed soon.
        let components = unsafe { self.components.try_borrow_unguarded().unwrap() };
        components.get(index).map(ComponentStorage::get)
    }

    /// Tries to get a mutable reference to
    /// the model of one element.
    ///
    /// Returns `None` is `index` is invalid.
    ///
    /// # Panics
    ///
    /// Panics when the same element was borrowed
    /// somewhere else.
    pub fn try_get_mut(&mut self, index: usize) -> Option<RefMut<'_, C>> {
        // Mark as modified
        if let Some(state) = self.model_state.get_mut().get_mut(index) {
            state.changed = true;
        }
        self.components
            .get_mut()
            .get_mut(index)
            .map(ComponentStorage::get_mut)
    }

    /// Provides a reference to the element at the given index.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics when the index is invalid or
    /// the same element was borrowed mutably somewhere else.
    pub fn get(&self, index: usize) -> Ref<'_, C> {
        self.try_get(index)
            .expect("Called `get` on an invalid index")
    }

    /// Provides a mutable reference to the element at the given index.
    ///
    /// Element at index 0 is the front of the queue.
    ///
    /// # Panics
    ///
    /// Panics when the index is invalid or
    /// the same element was borrowed somewhere else.
    pub fn get_mut(&mut self, index: usize) -> RefMut<'_, C> {
        self.try_get_mut(index)
            .expect("Called `get_mut` on an invalid index")
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

    /// Removes and returns the element at index from the [`FactoryVecDeque`].
    /// Returns [`None`] if index is out of bounds.
    ///
    /// Element at index 0 is the front of the queue.
    pub fn remove(&mut self, index: usize) -> Option<C> {
        let model_state = self.model_state.get_mut();
        let components = self.components.get_mut();

        model_state.remove(index);
        let component = components.remove(index);

        // Decrement the indexes of the following elements.
        for states in model_state.iter_mut().skip(index) {
            states.index.decrement();
        }

        if let Some(comp) = &component {
            if let Some(widget) = &comp.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }

        component.map(ComponentStorage::extract)
    }

    /// Returns the widget all components are attached to.
    pub fn widget(&self) -> &Widget {
        &self.widget
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
        let model_state = self.model_state.get_mut();
        let components = self.components.get_mut();

        let dyn_index = DynamicIndex::new(index);

        // Increment the indexes of the following elements.
        for states in model_state.iter_mut().skip(index) {
            states.index.increment();
        }

        let builder = FactoryBuilder::new(&dyn_index, init_params);

        components.insert(index, ComponentStorage::Builder(builder));
        model_state.insert(
            index,
            ModelStateValue {
                index: dyn_index,
                uid: self.uid_counter,
                changed: false,
            },
        );
        self.uid_counter += 1;
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
            let model_state = self.model_state.get_mut();
            let components = self.components.get_mut();

            model_state.swap(first, second);
            components.swap(first, second);

            // Update indexes.
            model_state[first].index.set_value(first);
            model_state[second].index.set_value(second);
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
            let model_state = self.model_state.get_mut();
            let components = self.components.get_mut();

            let elem = model_state.remove(current_position).unwrap();
            // Set new index
            elem.index.set_value(target);
            model_state.insert(target, elem);

            let comp = components.remove(current_position).unwrap();
            components.insert(target, comp);

            // Update indexes.
            if current_position > target {
                // Move down -> shift elements in between up.
                for state in model_state
                    .iter_mut()
                    .skip(target + 1)
                    .take(current_position - target)
                {
                    state.index.increment();
                }
            } else {
                // Move up -> shift elements in between down.
                for state in model_state
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

    /// Removes the first element from the [`FactoryVecDeque`] and returns it,
    /// or [`None`] if it is empty.
    pub fn pop_front(&mut self) -> Option<C> {
        self.remove(0)
    }
}

#[cfg(feature = "libadwaita")]
impl<C, ParentMsg> FactoryVecDeque<adw::TabView, C, ParentMsg>
where
    C: FactoryComponent<adw::TabView, ParentMsg> + Position<()>,
    C::Root: AsRef<gtk::Widget>,
{
    /// Apply external updates that happened between the last render.
    ///
    /// **YOU MUST NOT EDIT THE [`FactoryVecDeque`] BETWEEN CALLING
    /// [`render_change`] AND THIS METHOD. THIS MIGHT CAUSE UNDEFINED BEHAVIOR.
    pub fn apply_external_updates(&mut self) {
        let length = self.widget().n_pages();
        let mut hashes: Vec<u64> = Vec::with_capacity(length as usize);

        for i in 0..length {
            let page = self.widget.nth_page(i);
            let mut hasher = DefaultHasher::default();
            page.hash(&mut hasher);
            hashes.push(hasher.finish());
        }

        for (index, hash) in hashes.iter().enumerate() {
            let rendered_state = self.rendered_state.get_mut();

            if *hash
                != rendered_state
                    .get(index)
                    .map(|state| state.widget_hash)
                    .unwrap_or_default()
            {
                let old_position = rendered_state
                    .iter()
                    .position(|state| state.widget_hash == *hash)
                    .expect("A new widget was added");

                let elem = rendered_state.remove(old_position).unwrap();
                rendered_state.insert(index, elem);

                self.move_to(old_position, index);
            }
        }
    }
}
