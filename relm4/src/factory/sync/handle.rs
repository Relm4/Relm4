use std::fmt;

use crate::factory::{DataGuard, FactoryComponent, FactoryView};
use crate::Sender;

/// Don't allow public access to a [`FactoryHandle`].
///
/// It might be unsafe to extract `data` or `runtime`.
/// Inside this type, it is guaranteed that extracting `data` will drop `runtime` before to
/// comply with all required safety guarantees.
pub struct FactoryHandle<C: FactoryComponent> {
    pub(super) data: DataGuard<C, C::Widgets, C::Output>,
    pub root_widget: C::Root,
    pub returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
    pub input: Sender<C::Input>,
    pub notifier: Sender<()>,
}

impl<C: FactoryComponent> fmt::Debug for FactoryHandle<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FactoryHandle")
            .field("data", &"<FactoryComponent>")
            .field("root_widget", &self.root_widget)
            .field("input", &self.input)
            .field("notifier", &self.notifier)
            .finish()
    }
}
