use gtk::glib;

use crate::binding::Binding;

glib::wrapper! {
    pub struct BoolBinding(ObjectSubclass<imp::BoolBinding>);
}

impl Default for BoolBinding {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Binding for BoolBinding {
    type Target = bool;

    fn get(&self) -> Self::Target {
        self.value()
    }

    fn set(&self, value: Self::Target) {
        self.set_value(value)
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::{ParamSpec, Properties, Value};
    use gtk::subclass::prelude::ObjectImpl;
    use gtk::{
        glib,
        subclass::prelude::{DerivedObjectProperties, ObjectSubclass},
    };

    #[derive(Default, Properties, Debug)]
    #[properties(wrapper_type = super::BoolBinding)]
    pub struct BoolBinding {
        #[property(get, set)]
        value: RefCell<bool>,
    }

    impl ObjectImpl for BoolBinding {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BoolBinding {
        const NAME: &'static str = "BoolBinding";
        type Type = super::BoolBinding;
    }
}
