use super::ComponentParts;
use std::cell::{Ref, RefCell, RefMut};

/// Keeps track of a components model and view.
///
/// Borrowing the model and view will notify the component to check for updates.
#[derive(Debug)]
pub struct StateWatcher<Component, Widgets> {
    /// The models and widgets maintained by the component.
    pub(super) state: RefCell<ComponentParts<Component, Widgets>>,
    pub(super) notifier: flume::Sender<()>,
}

impl<Component, Widgets> StateWatcher<Component, Widgets> {
    /// Borrows the model and view of a component.
    pub fn get(&self) -> Ref<'_, ComponentParts<Component, Widgets>> {
        self.state.borrow()
    }

    /// Borrows the model and view of a component, and notifies the component to check for updates.
    pub fn get_mut(&self) -> RefMut<'_, ComponentParts<Component, Widgets>> {
        let _ = self.notifier.send(());
        self.state.borrow_mut()
    }
}
