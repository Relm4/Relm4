use glib::prelude::{IsA, ObjectExt};
use gtk::glib;

use crate::binding::Binding;

/// Trait that extends [`gtk::prelude::ObjectExt`].
pub trait RelmObjectExt {
    /// Runs the given function when the object is destroyed.
    fn on_destroy<F: FnOnce() + 'static>(&self, func: F);

    /// Bind a data binding to a property of an object.
    ///
    /// This is similar to [`glib::ObjectExt::bind_property`] and
    /// always bidirectional.
    fn add_binding<B: Binding>(&self, binding: &B, property_name: &str);

    /// Bind a data binding to a property of an object with
    /// uni-directional access, so values can only be written but are not synced
    /// in the other direction.
    fn add_write_only_binding<B: Binding>(&self, binding: &B, property_name: &str);
}

impl<T: IsA<glib::Object>> RelmObjectExt for T {
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
            .sync_create()
            .build();
    }

    fn add_write_only_binding<B: Binding>(&self, binding: &B, property_name: &str) {
        binding
            .bind_property(B::property_name(), self, property_name)
            .sync_create()
            .build();
    }
}
