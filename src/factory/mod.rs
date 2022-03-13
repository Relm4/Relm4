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

use std::cell::{Ref, RefMut};

use std::collections::VecDeque;

#[allow(missing_debug_implementations)]
/// A container similar to [`VecDeque`] that can be used to store
/// data associated with components that implement [`FactoryComponent`].
pub struct FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView + FactoryViewPlus,
    C: FactoryComponent<Widget, ParentMsg>,
    C::Root: AsRef<Widget::Children>,
    ParentMsg: 'static,
    Widget: 'static,
{
    widget: Widget,
    parent_sender: Sender<ParentMsg>,
    components: VecDeque<ComponentStorage<Widget, C, ParentMsg>>,
    model_state: VecDeque<ModelStateValue>,
    rendered_state: VecDeque<u16>,
    uid_counter: u16,
}

impl<Widget, C, ParentMsg> Drop for FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView + FactoryViewPlus,
    C: FactoryComponent<Widget, ParentMsg>,
    C::Root: AsRef<Widget::Children>,
{
    fn drop(&mut self) {
        for component in &self.components {
            if let Some(widget) = component.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }
    }
}

struct ModelStateValue {
    index: DynamicIndex,
    uid: u16,
    changed: bool,
}

impl<Widget, C, ParentMsg> FactoryVecDeque<Widget, C, ParentMsg>
where
    Widget: FactoryView + FactoryViewPlus,
    C: FactoryComponent<Widget, ParentMsg>,
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

    /// Updates the widgets according to the changes made to the factory.
    /// All updates accumulate until this method is called and are handled
    /// efficiently.
    ///
    /// For example, swapping two elements twice will only swap the data twice,
    /// but won't cause any UI updates.
    ///
    /// Also, only modified elements will be updated.
    pub fn render_changes(&mut self) {
        for (index, state) in self.model_state.iter().enumerate() {
            if state.uid == self.rendered_state.front().copied().unwrap_or_default() {
                // Remove item from previously rendered list
                self.rendered_state.pop_front();

                if state.changed {
                    // Update component
                    self.components[index].state_change_notify();
                }
                // else: nothing changed
            } else if let Some(rendered_index) =
                self.rendered_state.iter().position(|idx| *idx == state.uid)
            {
                // Remove item from previously rendered list
                self.rendered_state.remove(rendered_index);

                // Detach and re-attach item
                let widget = self.components[index].returned_widget().unwrap();
                if index == 0 {
                    self.widget.factory_move_start(widget);
                } else {
                    let previous_widget = self.components[index - 1].returned_widget().unwrap();
                    self.widget.factory_move_after(widget, previous_widget);
                }

                if state.changed {
                    // Update component
                    self.components[index].state_change_notify();
                }
            } else {
                // The element doesn't exist yet
                let insert_widget = self.components[index].widget();
                //let insert_widget = Widget::returned_widget_to_child(widget.as_ref());
                let returned_widget = if index == 0 {
                    self.widget.factory_prepend(insert_widget)
                } else {
                    let previous_widget = self.components[index - 1].returned_widget().unwrap();
                    self.widget
                        .factory_insert_after(insert_widget, previous_widget)
                };
                let component = self.components.remove(index).unwrap();
                let dyn_index = &self.model_state[index].index;
                let component = component
                    .launch(dyn_index, returned_widget, &self.parent_sender)
                    .unwrap();
                self.components.insert(index, component);
            }
        }
        // Set rendered state to the state of the model
        // because everything should be up-to-date now.
        self.rendered_state = self.model_state.iter().map(|s| s.uid).collect();
    }

    /// Returns the widget all components are attached to.
    pub fn widget(&self) -> &Widget {
        &self.widget
    }

    /// Appends an element at the end of the [`FactoryVecDeque`].
    pub fn push_back(&mut self, init_params: C::InitParams) {
        let index = self.model_state.len();
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
        for states in self.model_state.iter_mut().skip(index) {
            states.index.increment();
        }

        let builder = FactoryBuilder::new(&dyn_index, init_params);

        self.components
            .insert(index, ComponentStorage::Builder(builder));
        self.model_state.insert(
            index,
            ModelStateValue {
                index: dyn_index,
                uid: self.uid_counter,
                changed: false,
            },
        );
        self.uid_counter += 1;
    }

    /// Returns the number of elements in the [`FactoryVecDeque`].
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns true if the [`FactoryVecDeque`] is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
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
        if first != second {
            self.model_state.swap(first, second);

            // Update indexes.
            self.model_state[first].index.set_value(first);
            self.model_state[second].index.set_value(second);
        }

        self.components.swap(first, second);
    }

    /// Send a message to one of the elements.
    pub fn send(&self, index: usize, msg: C::Input) {
        self.components[index].send(msg)
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
        self.components.get(index).map(ComponentStorage::get)
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
        if let Some(state) = self.model_state.get_mut(index) {
            state.changed = true;
        }
        self.components.get(index).map(ComponentStorage::get_mut)
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
        if self.components.is_empty() {
            None
        } else {
            self.remove(self.components.len() - 1)
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
        self.model_state.remove(index);
        let component = self.components.remove(index);

        // Decrement the indexes of the following elements.
        for states in self.model_state.iter_mut().skip(index) {
            states.index.decrement();
        }

        if let Some(comp) = &component {
            if let Some(widget) = &comp.returned_widget() {
                self.widget.factory_remove(widget);
            }
        }

        component.map(ComponentStorage::extract)
    }
}
