use crate::factory::{DynamicIndex, FactoryView};
use crate::Sender;

use super::traits::AsyncFactoryComponent;
use super::AsyncFactoryBuilder;
use super::AsyncFactoryHandle;

#[derive(Debug)]
pub enum AsyncComponentStorage<C: AsyncFactoryComponent>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    Builder(AsyncFactoryBuilder<C>),
    Final(AsyncFactoryHandle<C>),
}

impl<C: AsyncFactoryComponent> AsyncComponentStorage<C>
where
    <C::ParentWidget as FactoryView>::ReturnedWidget: Clone,
{
    pub fn get(&self) -> Option<&C> {
        match self {
            Self::Builder(_) => None,
            Self::Final(handle) => handle.data.get(),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut C> {
        match self {
            Self::Builder(_) => None,
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

    pub fn extract(self) -> Option<C> {
        match self {
            Self::Builder(_) => None,
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
