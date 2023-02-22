use gtk::glib;

use crate::binding::Binding;

glib::wrapper! {
    pub struct IntBinding(ObjectSubclass<imp::IntBinding>);
}

impl Default for IntBinding {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Binding for IntBinding {
    type Target = i32;

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
    #[properties(wrapper_type = super::IntBinding)]
    pub struct IntBinding {
        #[property(get, set)]
        value: RefCell<i32>,
    }

    impl ObjectImpl for IntBinding {
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
    impl ObjectSubclass for IntBinding {
        const NAME: &'static str = "IntBinding";
        type Type = super::IntBinding;
    }
}
