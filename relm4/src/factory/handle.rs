use super::data_guard::DataGuard;
use super::FactoryComponent;

use std::fmt;

use super::FactoryView;
use crate::Sender;

/// Don't allow public access to a [`FactoryHandle`].
///
/// It might be unsafe to extract `data` or `runtime`.
/// Inside this type, it is guaranteed that extracting `data` will drop `runtime` before to
/// comply with all required safety guarantees.
pub(super) struct FactoryHandle<Widget, C: FactoryComponent<Widget, ParentMsg>, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    pub(super) data: DataGuard<C>,
    pub(super) root_widget: C::Root,
    pub(super) returned_widget: Widget::ReturnedWidget,
    pub(super) input: Sender<C::Input>,
    pub(super) notifier: Sender<()>,
}

impl<Widget, C, ParentMsg> FactoryHandle<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
}

impl<Widget, C, ParentMsg> fmt::Debug for FactoryHandle<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FactoryHandle")
            .field("data", &self.data)
            .field("root_widget", &self.root_widget)
            .field("input", &self.input)
            .field("notifier", &self.notifier)
            .finish()
    }
}
