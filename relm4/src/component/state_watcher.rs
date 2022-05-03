use super::{Component, ComponentParts};

use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{self, Debug};

/// Keeps track of a components model and view.
///
/// Borrowing the model and view will notify the component to check for updates.
pub struct StateWatcher<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: RefCell<ComponentParts<C>>,
    pub(super) notifier: flume::Sender<()>,
}

impl<C: Component> StateWatcher<C> {
    /// Borrows the model and view of a component.
    pub fn get(&self) -> Ref<'_, ComponentParts<C>> {
        self.state.borrow()
    }

    /// Borrows the model and view of a component, and notifies the component to check for updates.
    pub fn get_mut(&self) -> RefMut<'_, ComponentParts<C>> {
        let _ = self.notifier.send(());
        self.state.borrow_mut()
    }
}

impl<C: Component> Debug for StateWatcher<C>
where
    C: Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateWatcher")
            .field("state", &self.state)
            .field("notifier", &self.notifier)
            .finish()
    }
}
