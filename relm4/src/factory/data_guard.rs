use std::{cell::Cell, mem::ManuallyDrop, rc::Rc};

use futures::Future;
use gtk::glib;

/// A type that will drop a runtime behind a shared reference
/// when it is dropped.
pub(super) struct DataGuard<Data: std::fmt::Debug> {
    data: Box<Data>,
    rt: RuntimeDropper,
}

impl<Data: std::fmt::Debug> DataGuard<Data> {
    pub(super) fn new<F, Fut>(data: Box<Data>, f: F) -> (Self, Rc<Cell<Option<glib::SourceId>>>)
    where
        Fut: Future<Output = ()> + 'static,
        F: FnOnce(ManuallyDrop<Box<Data>>) -> Fut,
    {
        // Duplicate the references to `data`
        // # SAFETY
        // This is safe because:
        // 1. The first reference never calls the destructor (being wrapped in ManuallyDrop)
        // 2. The first reference is always dropped first. This is guaranteed by types like
        //    `RuntimeDropper` and `FactoryHandle` that wrap the data and the runtime ID
        //    in a safe API that makes sure the runtime (and with it the first reference) is
        //    dropped before the second reference is dropped or extracted.
        // 3. The second reference can only be extracted or dropped AFTER the first one
        //    was dropped. The second reference can then safely behave like a normal `Box<C>`.
        let (data, model_data) = unsafe {
            let raw = Box::into_raw(data);
            let data = Box::from_raw(raw);
            let runtime_data = ManuallyDrop::new(Box::from_raw(raw));
            (data, runtime_data)
        };

        let future = f(model_data);
        let rt = Rc::new(Cell::new(Some(crate::spawn_local(future))));
        let rt_ref = Rc::clone(&rt);
        (
            Self {
                data,
                rt: RuntimeDropper(rt),
            },
            rt_ref,
        )
    }

    pub(super) fn get(&self) -> &Data {
        &self.data
    }

    pub(super) fn get_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub(super) fn into_inner(self) -> Data {
        drop(self.rt);
        *self.data
    }
}

struct RuntimeDropper(Rc<Cell<Option<glib::SourceId>>>);

impl Drop for RuntimeDropper {
    fn drop(&mut self) {
        if let Some(id) = self.0.take() {
            id.remove();
        }
    }
}

impl<Data: std::fmt::Debug> std::fmt::Debug for DataGuard<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataGuard")
            .field("data", &self.data)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use gtk::glib::MainContext;

    use super::DataGuard;

    #[derive(Debug)]
    struct DontDropBelow4(u8);

    impl DontDropBelow4 {
        fn value(&self) -> u8 {
            self.0
        }

        fn add(&mut self) {
            self.0 += 1
        }
    }

    impl Drop for DontDropBelow4 {
        fn drop(&mut self) {
            if self.0 < 4 {
                panic!()
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_drop_panic() {
        let _data = Box::new(DontDropBelow4(0_u8));
    }

    #[test]
    fn test_data_guard() {
        let data = Box::new(DontDropBelow4(0_u8));
        let (tx, rx) = flume::bounded(3);

        let main_ctx = MainContext::default();

        let (data, rt) = DataGuard::new(data, |mut rt_data| async move {
            while let Ok(_) = rx.recv_async().await {
                rt_data.add();
            }
        });

        main_ctx.iteration(false);
        assert_eq!(data.get().value(), 0);

        tx.send(()).unwrap();
        tx.send(()).unwrap();
        tx.send(()).unwrap();
        main_ctx.iteration(false);

        assert_eq!(data.get().value(), 3);

        let mut data = data.into_inner();
        // If destructor was called, it should have paniced by now
        assert_eq!(data.value(), 3);
        assert!(rt.take().is_none());

        // Make sure the destructor doesn't panic
        data.add();
        assert_eq!(data.value(), 4);

        tx.send(()).unwrap_err();
        tx.send(()).unwrap_err();
        tx.send(()).unwrap_err();
        main_ctx.iteration(false);
    }
}
