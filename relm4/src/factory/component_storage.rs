use crate::factory::{DynamicIndex, FactoryBuilder, FactoryComponent, FactoryHandle, FactoryView};
use crate::Sender;

use std::cell::{Ref, RefMut};
use std::rc::Rc;

#[allow(missing_debug_implementations)]
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
    pub(super) fn get(&self) -> Ref<'_, C> {
        match self {
            Self::Builder(builder) => builder.data.borrow(),
            Self::Final(handle) => handle.data.borrow(),
        }
    }

    pub(super) fn get_mut(&mut self) -> RefMut<'_, C> {
        match self {
            Self::Builder(builder) => builder.data.borrow_mut(),
            Self::Final(handle) => handle.data.borrow_mut(),
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
            Self::Final(handle) => handle.input.send(msg),
        }
    }

    pub(super) fn state_change_notify(&self) {
        if let Self::Final(handle) = self {
            handle.notifier.send(());
        }
    }

    pub(super) fn extract(self) -> C {
        match self {
            Self::Builder(builder) => Rc::try_unwrap(builder.data).unwrap().into_inner(),
            Self::Final(handle) => {
                if let Some(id) = handle.burner.runtime_id.borrow_mut().take() {
                    id.remove();
                }
                Rc::try_unwrap(handle.data).unwrap().into_inner()
            }
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
