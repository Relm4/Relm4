use super::runtime_dropper::RuntimeDropper;
use super::FactoryComponent;

use std::fmt;

use super::FactoryView;
use crate::Sender;

pub(super) struct FactoryHandleData<Widget, C: FactoryComponent<Widget, ParentMsg>, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    pub(super) data: Box<C>,
    pub(super) root_widget: C::Root,
    pub(super) returned_widget: Widget::ReturnedWidget,
    pub(super) input: Sender<C::Input>,
    pub(super) notifier: Sender<()>,
    pub(super) runtime: RuntimeDropper,
}

pub(super) struct FactoryHandle<Widget, C: FactoryComponent<Widget, ParentMsg>, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    data: Box<C>,
    pub(super) root_widget: C::Root,
    pub(super) returned_widget: Widget::ReturnedWidget,
    input: Sender<C::Input>,
    pub(super) notifier: Sender<()>,
    runtime: RuntimeDropper,
}

impl<Widget, C, ParentMsg> FactoryHandle<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    pub(super) fn new(data: FactoryHandleData<Widget, C, ParentMsg>) -> Self {
        let FactoryHandleData {
            data,
            root_widget,
            returned_widget,
            input,
            notifier,
            runtime,
        } = data;
        Self {
            data,
            root_widget,
            returned_widget,
            input,
            notifier,
            runtime,
        }
    }

    pub(super) fn get_data(&self) -> &C {
        &self.data
    }

    pub(super) fn send(&self, msg: C::Input) {
        self.input.send(msg)
    }

    pub(super) fn get_mut_data(&mut self) -> &mut C {
        &mut self.data
    }

    pub(super) fn into_data(self) -> C {
        drop(self.runtime);
        *self.data
    }
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
