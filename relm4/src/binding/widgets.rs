use crate::RelmObjectExt;

use super::{Binding, ConnectBinding};

macro_rules! impl_connect_binding {
    ($ty:ty, $target:ty, $primary_prop:literal, $mod:ident) => {
        impl_connect_binding!($ty, $target, $primary_prop, $mod, gtk::glib::Object::new());
    };
    ($ty:ty, $target:ty, $primary_prop:literal, $mod:ident, $test_init:expr) => {
        #[doc = "Create a data binding to the primary property `"]
        #[doc = $primary_prop]
        #[doc = "` with type [`"]
        #[doc = stringify!($target)]
        #[doc = "`]."]
        impl ConnectBinding for $ty {
            type Target = $target;

            fn bind<B: Binding<Target = Self::Target>>(&self, binding: &B) {
                self.add_binding(binding, $primary_prop);
            }
        }

        #[cfg(test)]
        mod $mod {
            use gtk::prelude::ObjectExt;

            #[gtk::test]
            /// Test whether the property name and type are correct.
            fn test() {
                let obj: $ty = $test_init;
                let data: $target = Default::default();
                obj.set_property($primary_prop, data);
            }
        }
    };
}

// bool bindings
impl_connect_binding!(gtk::ToggleButton, bool, "active", toggle_button);
impl_connect_binding!(gtk::Switch, bool, "active", switch);
impl_connect_binding!(gtk::Spinner, bool, "spinning", spinner);
impl_connect_binding!(gtk::Popover, bool, "visible", popover);
impl_connect_binding!(gtk::Revealer, bool, "reveal-child", revealer);

// f64 bindings
impl_connect_binding!(gtk::SpinButton, f64, "value", spin_button);
impl_connect_binding!(gtk::Adjustment, f64, "value", adjustment);
impl_connect_binding!(gtk::ScaleButton, f64, "value", scale_button);

// String bindings
impl_connect_binding!(gtk::Label, String, "label", label);
impl_connect_binding!(gtk::Button, String, "label", button);
impl_connect_binding!(gtk::LinkButton, String, "uri", link_button);
impl_connect_binding!(gtk::MenuButton, String, "label", menu_button);
impl_connect_binding!(gtk::Image, String, "icon-name", image);
impl_connect_binding!(gtk::StackPage, String, "name", stack_page, {
    let stack = gtk::Stack::default();
    stack.add_child(&gtk::Label::default())
});
