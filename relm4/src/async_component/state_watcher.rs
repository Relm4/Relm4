use super::{AsyncComponent, AsyncComponentParts};

use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{self, Debug};

/// Keeps track of a components model and view.
///
/// Borrowing the model and view will notify the component to check for updates.
pub struct StateWatcher<C: AsyncComponent> {
    /// The models and widgets maintained by the component.
    pub(super) state: RefCell<AsyncComponentParts<C>>,
    pub(super) notifier: flume::Sender<()>,
}

impl<C: AsyncComponent> StateWatcher<C> {
    /// Borrows the model and view of a component.
    pub fn get(&self) -> Ref<'_, AsyncComponentParts<C>> {
        self.state.borrow()
    }

    /// Borrows the model and view of a component, and notifies the component to check for updates.
    pub fn get_mut(&self) -> RefMut<'_, AsyncComponentParts<C>> {
        let _ = self.notifier.send(());
        self.state.borrow_mut()
    }
}

impl<C> Debug for StateWatcher<C>
where
    C: AsyncComponent + Debug,
    C::Widgets: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateWatcher")
            .field("state", &self.state)
            .field("notifier", &self.notifier)
            .finish()
    }
}
