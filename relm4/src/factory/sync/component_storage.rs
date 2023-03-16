use crate::factory::{DynamicIndex, FactoryComponent, FactoryView};
use crate::Sender;

use super::{FactoryBuilder, FactoryHandle};

#[derive(Debug)]
pub enum ComponentStorage<C: FactoryComponent> {
    Builder(FactoryBuilder<C>),
    Final(FactoryHandle<C>),
}

impl<C: FactoryComponent> ComponentStorage<C> {
    pub const fn get(&self) -> &C {
        match self {
            Self::Builder(builder) => &builder.data,
            Self::Final(handle) => handle.data.get(),
        }
    }

    pub fn get_mut(&mut self) -> &mut C {
        match self {
            Self::Builder(builder) => &mut builder.data,
            Self::Final(handle) => handle.data.get_mut(),
        }
    }

    pub const fn widget(&self) -> &C::Root {
        match self {
            Self::Builder(builder) => &builder.root_widget,
            Self::Final(handle) => &handle.root_widget,
        }
    }

    pub fn send(&self, msg: C::Input) {
        match self {
            Self::Builder(builder) => builder.component_sender.input(msg),
            Self::Final(handle) => handle.input.send(msg).unwrap(),
        }
    }

    pub fn state_change_notify(&self) {
        if let Self::Final(handle) = self {
            handle.notifier.send(()).unwrap();
        }
    }

    pub fn extract(self) -> C {
        match self {
            Self::Builder(builder) => *builder.data,
            Self::Final(handle) => handle.data.into_inner(),
        }
    }

    pub fn launch(
        self,
        index: &DynamicIndex,
        returned_widget: <C::ParentWidget as FactoryView>::ReturnedWidget,
        parent_sender: &Sender<C::ParentInput>,
    ) -> Option<Self> {
        if let Self::Builder(builder) = self {
            Some(Self::Final(builder.launch(
                index,
                returned_widget,
                parent_sender,
                C::output_to_parent_input,
            )))
        } else {
            None
        }
    }

    pub const fn returned_widget(
        &self,
    ) -> Option<&<C::ParentWidget as FactoryView>::ReturnedWidget> {
        if let Self::Final(handle) = self {
            Some(&handle.returned_widget)
        } else {
            None
        }
    }
}
