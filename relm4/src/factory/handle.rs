use super::FactoryComponent;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::FactoryView;
use crate::Sender;
use gtk::glib;

pub(super) struct FactoryHandle<Widget, C: FactoryComponent<Widget, ParentMsg>, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    pub(super) data: Rc<RefCell<C>>,
    pub(super) root_widget: C::Root,
    pub(super) returned_widget: Widget::ReturnedWidget,
    pub(super) input: Sender<C::Input>,
    pub(super) notifier: Sender<()>,
    pub(super) runtime_id: Rc<RefCell<Option<glib::SourceId>>>,
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
            .field("runtime_id", &self.runtime_id)
            .finish()
    }
}
