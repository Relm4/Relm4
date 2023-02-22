use glib::prelude::ObjectExt;
use gtk::glib;

use crate::binding::Binding;

/// Trait that extends [`gtk::prelude::ObjectExt`].
pub trait RelmObjectExt {
    /// Runs the given function when the object is destroyed.
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F);

    fn add_binding<B: Binding>(&self, binding: &B, property_name: &str);
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

    fn add_binding<B: Binding>(&self, binding: &B, property_name: &str) {
        binding
            .bind_property(B::property_name(), self, property_name)
            .bidirectional()
            .build();
    }
}
