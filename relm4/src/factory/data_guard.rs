use std::mem::ManuallyDrop;

use futures::Future;
use gtk::glib;

use crate::{shutdown::ShutdownSender, Sender};

type DynShutdownFn<C, Widgets, Output> = dyn Fn(&mut C, &mut Widgets, Sender<Output>);

/// # SAFETY
///
/// This type is a safe wrapper that prevent's misuse,
/// except if you move the data passed to the runtime outside
/// of the runtime (through senders for example).
pub(super) struct DataGuard<C, Widgets, Output> {
    data: Box<C>,
    widgets: Box<Widgets>,
    rt_dropper: RuntimeDropper,
    output_sender: Sender<Output>,
    shutdown_notifier: ShutdownSender,
    shutdown_fn: Box<DynShutdownFn<C, Widgets, Output>>,
}

impl<C, Widgets, Output> DataGuard<C, Widgets, Output> {
    /// DO NOT MOVE THE DATA PASSED TO THE CLOSURE OUTSIDE OF THE RUNTIME!
    /// SAFETY IS ONLY GUARANTEED BECAUSE THE DATA IS BOUND TO THE LIFETIME OF THE RUNTIME!
    pub(super) fn new<F, Fut, ShutdownFn>(
        data: Box<C>,
        widgets: Box<Widgets>,
        shutdown_notifier: ShutdownSender,
        output_sender: Sender<Output>,
        f: F,
        shutdown_fn: ShutdownFn,
    ) -> Self
    where
        Fut: Future<Output = ()> + 'static,
        F: FnOnce(ManuallyDrop<Box<C>>, ManuallyDrop<Box<Widgets>>) -> Fut,
        ShutdownFn: Fn(&mut C, &mut Widgets, Sender<Output>) + 'static,
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
        let (data, runtime_data) = unsafe {
            let raw = Box::into_raw(data);
            let data = Box::from_raw(raw);
            let runtime_data = ManuallyDrop::new(Box::from_raw(raw));
            (data, runtime_data)
        };
        let (widgets, runtime_widgets) = unsafe {
            let raw = Box::into_raw(widgets);
            let widgets = Box::from_raw(raw);
            let runtime_widgets = ManuallyDrop::new(Box::from_raw(raw));
            (widgets, runtime_widgets)
        };

        let future = f(runtime_data, runtime_widgets);
        let rt_dropper = RuntimeDropper(Some(crate::spawn_local(future).into_source_id().unwrap()));
        let shutdown_fn = Box::new(shutdown_fn);

        Self {
            data,
            widgets,
            output_sender,
            shutdown_notifier,
            rt_dropper,
            shutdown_fn,
        }
    }

    pub(super) const fn get(&self) -> &C {
        &self.data
    }

    pub(super) fn get_mut(&mut self) -> &mut C {
        &mut self.data
    }

    pub(super) fn into_inner(mut self) -> C {
        drop(self.rt_dropper);
        self.shutdown_notifier.shutdown();
        (self.shutdown_fn)(
            &mut self.data,
            &mut self.widgets,
            self.output_sender.clone(),
        );
        drop(self.widgets);
        *self.data
    }
}

impl<C, Widgets, Output> std::fmt::Debug for DataGuard<C, Widgets, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataGuard")
            .field("data", &"<Data>")
            .field("widgets", &"<Widgets>")
            .field("rt_dropper", &self.rt_dropper)
            .field("output_sender", &self.output_sender)
            .field("shutdown_notifier", &self.shutdown_notifier)
            .field("shutdown_fn", &"<shutdown fn>")
            .finish()
    }
}

#[derive(Debug)]
struct RuntimeDropper(Option<glib::SourceId>);

/// A type that will drop a runtime behind a shared reference
/// when it is dropped.
impl Drop for RuntimeDropper {
    fn drop(&mut self) {
        if let Some(id) = self.0.take() {
            id.remove();
        }
    }
}

#[cfg(test)]
mod test {
    use gtk::glib::MainContext;

    use crate::{prelude::FactoryComponent, shutdown};

    use super::DataGuard;

    #[derive(Debug)]
    struct DontDropBelow4(u8);

    impl DontDropBelow4 {
        fn value(&self) -> u8 {
            self.0
        }

        fn add(&mut self) {
            self.0 += 1;
        }
    }

    impl Drop for DontDropBelow4 {
        fn drop(&mut self) {
            if self.0 < 4 {
                panic!()
            }
        }
    }

    impl FactoryComponent for DontDropBelow4 {
        type ParentWidget = gtk::Box;
        type CommandOutput = ();
        type Input = ();
        type Output = ();
        type Init = ();
        type Root = gtk::Box;
        type Widgets = ();
        type Index = crate::prelude::DynamicIndex;

        fn init_model(
            _: Self::Init,
            _: &crate::prelude::DynamicIndex,
            _: crate::prelude::FactorySender<Self>,
        ) -> Self {
            Self(0)
        }

        fn init_root(&self) -> Self::Root {
            gtk::Box::default()
        }

        fn init_widgets(
            &mut self,
            _: &crate::prelude::DynamicIndex,
            _: &Self::Root,
            _: &<Self::ParentWidget as crate::factory::FactoryView>::ReturnedWidget,
            _: crate::prelude::FactorySender<Self>,
        ) -> Self::Widgets {
        }

        fn shutdown(&mut self, _: &mut Self::Widgets, _: crate::Sender<Self::Output>) {
            self.add();
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
        let widgets = Box::new(());
        let (shutdown_notifier, mut shutdown_receiver) = shutdown::channel();
        let (output_sender, _) = crate::channel();
        let (tx, rx) = flume::unbounded();

        let data = DataGuard::new(
            data,
            widgets,
            shutdown_notifier,
            output_sender,
            |mut rt_data, _| async move {
                while (rx.recv_async().await).is_ok() {
                    rt_data.add();
                }
            },
            DontDropBelow4::shutdown,
        );

        let main_ctx = MainContext::default();

        main_ctx.iteration(false);
        assert_eq!(data.get().value(), 0);

        tx.send(()).unwrap();
        tx.send(()).unwrap();
        main_ctx.iteration(false);

        assert_eq!(data.get().value(), 2);

        let mut data = data.into_inner();
        // Make sure the shutdown is called with yet another increment.
        assert_eq!(data.value(), 3);
        // Make sure the shutdown receiver was notified.
        shutdown_receiver.try_recv().unwrap();

        main_ctx.iteration(false);

        // Make sure the destructor doesn't panic
        data.add();
        assert_eq!(data.value(), 4);

        tx.send(()).unwrap_err();
        main_ctx.iteration(false);
    }
}
