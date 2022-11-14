use std::cell::RefCell;

use once_cell::unsync::OnceCell;

use super::AsyncFactoryComponent;
use crate::factory::data_guard::DataGuard;
use crate::Receiver;

type DataReceiver<T> = RefCell<Option<Receiver<T>>>;

pub(super) struct AsyncData<C: AsyncFactoryComponent> {
    future: DataReceiver<DataGuard<C, C::Widgets, C::Output>>,
    data: OnceCell<DataGuard<C, C::Widgets, C::Output>>,
}

impl<C: AsyncFactoryComponent> AsyncData<C> {
    pub(super) fn new(data: Receiver<DataGuard<C, C::Widgets, C::Output>>) -> Self {
        Self {
            future: RefCell::new(Some(data)),
            data: OnceCell::new(),
        }
    }
}

impl<C: AsyncFactoryComponent> AsyncData<C> {
    pub(super) fn get(&self) -> Option<&C> {
        self.update();
        self.data.get().map(|g| g.get())
    }

    pub(super) fn get_mut(&mut self) -> Option<&mut C> {
        self.update();
        self.data.get_mut().map(|g| g.get_mut())
    }

    pub(super) fn into_inner(self) -> Option<C> {
        self.update();
        self.data.into_inner().map(|g| g.into_inner())
    }

    fn update(&self) {
        let future = &mut *self.future.borrow_mut();
        if future.is_some() {
            if let Ok(data) = future.as_ref().unwrap().0.try_recv() {
                *future = None;
                self.data.set(data).ok().unwrap();
            }
        }
    }
}
