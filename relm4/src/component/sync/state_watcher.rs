use crate::{Component, ComponentParts, ShutdownOnDrop};

use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{self, Debug};
use std::rc::Rc;

/// Keeps track of a components model and view.
///
/// Borrowing the model and view will notify the component to check for updates.
pub struct StateWatcher<C: Component> {
    /// The models and widgets maintained by the component.
    pub(super) state: Rc<RefCell<ComponentParts<C>>>,
    pub(super) notifier: crate::Sender<()>,
    pub(super) shutdown_on_drop: ShutdownOnDrop,
}

impl<C: Component> StateWatcher<C> {
    /// Borrows the model and view of a component.
    #[must_use]
    pub fn get(&self) -> Ref<'_, ComponentParts<C>> {
        self.state.borrow()
    }

    /// Borrows the model and view of a component, and notifies the component to check for updates.
    #[must_use]
    pub fn get_mut(&self) -> RefMut<'_, ComponentParts<C>> {
        self.notifier.send(());
        self.state.borrow_mut()
    }

    pub(super) fn detach_runtime(&mut self) {
        self.shutdown_on_drop.deactivate()
    }
}

impl<C> Debug for StateWatcher<C>
where
    C: Component + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateWatcher")
            .field("state", &self.state)
            .field("notifier", &self.notifier)
            .field("shutdown_on_drop", &self.shutdown_on_drop)
            .finish()
    }
}
