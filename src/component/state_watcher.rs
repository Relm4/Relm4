use super::Fuselage;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use tokio::sync::Notify;

/// Keeps track of a components model and view.
///
/// Borrowing the model and view will notify the component to check for updates.
#[derive(Debug)]
pub struct StateWatcher<Component, Widgets> {
    /// The models and widgets maintained by the component.
    pub(super) state: RefCell<Fuselage<Component, Widgets>>,
    pub(super) notifier: Rc<Notify>,
}

impl<Component, Widgets> StateWatcher<Component, Widgets> {
    /// Borrows the model and view of a component.
    pub fn get(&self) -> Ref<'_, Fuselage<Component, Widgets>> {
        self.state.borrow()
    }

    /// Borrows the model and view of a component, and notifies the component to check for updates.
    pub fn get_mut(&self) -> RefMut<'_, Fuselage<Component, Widgets>> {
        self.notifier.notify_one();
        self.state.borrow_mut()
    }
}
