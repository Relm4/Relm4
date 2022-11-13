use glib::prelude::ObjectExt;
use gtk::glib;

/// Trait that extends [`gtk::prelude::ObjectExt`].
pub trait RelmObjectExt {
    /// Runs the given function when the object is destroyed.
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F);
}

impl<T: glib::IsA<glib::Object>> RelmObjectExt for T {
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F) {
        let func = std::cell::RefCell::new(Some(func));
        self.as_ref().add_weak_ref_notify_local(move || {
            if let Some(func) = func.take() {
                func();
            }
        });
    }
}
