use crate::factory::{
    builder::FactoryBuilder, handle::FactoryHandle, DynamicIndex, FactoryComponent, FactoryView,
};
use crate::Sender;

use std::fmt::Debug;

#[derive(Debug)]
pub(super) enum ComponentStorage<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
    ParentMsg: 'static,
    Widget: 'static,
{
    Builder(FactoryBuilder<Widget, C, ParentMsg>),
    Final(FactoryHandle<Widget, C, ParentMsg>),
}

impl<Widget, C, ParentMsg> ComponentStorage<Widget, C, ParentMsg>
where
    Widget: FactoryView,
    C: FactoryComponent<Widget, ParentMsg>,
{
    pub(super) fn get(&self) -> &C {
        match self {
            Self::Builder(builder) => &builder.data,
            Self::Final(handle) => handle.get_data(),
        }
    }

    pub(super) fn get_mut(&mut self) -> &mut C {
        match self {
            Self::Builder(builder) => &mut builder.data,
            Self::Final(handle) => handle.get_mut_data(),
        }
    }

    pub(super) fn widget(&self) -> &C::Root {
        match self {
            Self::Builder(builder) => &builder.root_widget,
            Self::Final(handle) => &handle.root_widget,
        }
    }

    pub(super) fn send(&self, msg: C::Input) {
        match self {
            Self::Builder(builder) => builder.input_tx.send(msg),
            Self::Final(handle) => handle.send(msg),
        }
    }

    pub(super) fn state_change_notify(&self) {
        if let Self::Final(handle) = self {
            handle.notifier.send(());
        }
    }

    pub(super) fn extract(self) -> C {
        match self {
            Self::Builder(builder) => *builder.data,
            Self::Final(handle) => handle.into_data(),
        }
    }

    pub(super) fn launch(
        self,
        index: &DynamicIndex,
        returned_widget: Widget::ReturnedWidget,
        parent_sender: &Sender<ParentMsg>,
    ) -> Option<Self> {
        if let Self::Builder(builder) = self {
            Some(Self::Final(builder.launch(
                index,
                returned_widget,
                parent_sender,
                C::output_to_parent_msg,
            )))
        } else {
            None
        }
    }

    pub(super) fn returned_widget(&self) -> Option<&Widget::ReturnedWidget> {
        if let Self::Final(handle) = self {
            Some(&handle.returned_widget)
        } else {
            None
        }
    }
}
