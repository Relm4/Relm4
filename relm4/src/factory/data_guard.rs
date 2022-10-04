use std::{cell::Cell, mem::ManuallyDrop, rc::Rc};

use futures::Future;
use gtk::glib;

/// # SAFETY
///
/// This type is a safe wrapper that prevent's misuse,
/// except if you move the data passed to the runtime outside
/// of the runtime (through senders for example).
pub(super) struct DataGuard<Data> {
    data: Box<Data>,
    rt: RuntimeDropper,
}

impl<Data> DataGuard<Data> {
    /// DO NOT MOVE THE DATA PASSED TO THE CLOSURE OUTSIDE OF THE RUNTIME!
    /// SAFETY IS ONLY GUARANTEED BECAUSE THE DATA IS BOUND TO THE LIFETIME OF THE RUNTIME!
    pub(super) fn new<F, Fut>(data: Box<Data>, f: F) -> (Self, Rc<Cell<Option<glib::SourceId>>>)
    where
        Fut: Future<Output = ()> + 'static,
        F: FnOnce(ManuallyDrop<Box<Data>>) -> Fut,
    {
        // Duplicate the references to `data`
        //
        // # SAFETY
        //
        // This is safe because:
        // 1. The first reference never calls the destructor (being wrapped in ManuallyDrop)
        // 2. The first reference is always dropped first. This is guaranteed by types like
        //    `RuntimeDropper` and `FactoryHandle` that wrap the data and the runtime ID
        //    in a safe API that makes sure the runtime (and with it the first reference) is
        //    dropped before the second reference is dropped or extracted.
        // 3. The second reference can only be extracted or dropped AFTER the first one
        //    was dropped. The second reference can then safely behave like a normal `Box<C>`.
        //
        // Unsoundness only occurs when data that was moved into the runtime is moved out on
        // purpose. This would allow the first reference to outlive the second one, becoming
        // a dangling pointer.
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

    pub(super) const fn get(&self) -> &Data {
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

/// A type that will drop a runtime behind a shared reference
/// when it is dropped.
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

    #[gtk::test]
    fn test_data_guard_drop() {
        let data = Box::new(DontDropBelow4(0_u8));
        let (tx, rx) = flume::unbounded();

        let main_ctx = MainContext::default();

        let (data, rt) = DataGuard::new(data, |mut rt_data| async move {
            while (rx.recv_async().await).is_ok() {
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
        // If destructor was called, it should have panicked by now
        assert_eq!(data.value(), 3);
        assert!(rt.take().is_none());

        main_ctx.iteration(false);

        // Make sure the destructor doesn't panic
        data.add();
        assert_eq!(data.value(), 4);

        tx.send(()).unwrap_err();
        main_ctx.iteration(false);
    }

    #[gtk::test]
    fn test_data_guard_rt_kill() {
        let data = Box::new(DontDropBelow4(0_u8));
        let (tx, rx) = flume::unbounded();

        let main_ctx = MainContext::default();

        let (data, rt) = DataGuard::new(data, |mut rt_data| async move {
            while (rx.recv_async().await).is_ok() {
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

        // Manually drop the runtime from outside
        rt.take().unwrap().remove();

        // Value shouldn't change
        tx.send(()).unwrap_err();
        main_ctx.iteration(false);
        assert_eq!(data.get().value(), 3);

        let mut data = data.into_inner();
        // If destructor was called, it should have panicked by now
        assert_eq!(data.value(), 3);
        assert!(rt.take().is_none());

        main_ctx.iteration(false);

        // Make sure the destructor doesn't panic
        data.add();
        assert_eq!(data.value(), 4);

        tx.send(()).unwrap_err();
        main_ctx.iteration(false);
    }
}
