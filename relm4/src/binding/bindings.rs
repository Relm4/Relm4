use gtk::glib;

use crate::binding::Binding;

macro_rules! binding {
    ($name:ident, $obj_name:literal, $ty:ty, $mod:ident) => {
        glib::wrapper! {
            #[doc = "A data binding storing a value of type [`"]
            #[doc = stringify!($ty)]
            #[doc = "`]"]
            pub struct $name(ObjectSubclass<$mod::$name>);
        }

        impl $name {
            #[doc = "Create a new [`"]
            #[doc = stringify!($name)]
            #[doc = "`]."]
            pub fn new<T: Into<$ty>>(value: T) -> Self {
                let this: Self = glib::Object::new();
                this.set_value(value.into());
                this
            }
        }

        impl Default for $name {
            fn default() -> Self {
                glib::Object::new()
            }
        }

        impl Binding for $name {
            type Target = $ty;

            fn get(&self) -> Self::Target {
                self.value()
            }

            fn set(&self, value: Self::Target) {
                self.set_value(value)
            }
        }

        #[allow(missing_docs)]
        #[allow(clippy::must_use_candidate)]
        mod $mod {
            use std::cell::RefCell;

            use glib::prelude::*;
            use glib::{ParamSpec, Properties, Value};
            use gtk::subclass::prelude::ObjectImpl;
            use gtk::{
                glib,
                subclass::prelude::{DerivedObjectProperties, ObjectSubclass},
            };

            #[derive(Default, Properties, Debug)]
            #[properties(wrapper_type = super::$name)]
            /// Inner type of the data binding.
            pub struct $name {
                #[property(get, set)]
                /// The primary value.
                value: RefCell<$ty>,
            }

            impl ObjectImpl for $name {
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
            impl ObjectSubclass for $name {
                const NAME: &'static str = $obj_name;
                type Type = super::$name;
            }
        }
    };
}

// Bool
binding!(BoolBinding, "BoolBinding", bool, imp_bool);

// Integers
binding!(U64Binding, "U64Binding", u64, imp_u64);
binding!(I64Binding, "I64Binding", i64, imp_i64);
binding!(U32Binding, "U32Binding", u32, imp_u32);
binding!(I32Binding, "I32Binding", i32, imp_i32);
binding!(U8Binding, "U8Binding", u8, imp_u8);
binding!(I8Binding, "I8Binding", i8, imp_i8);

// Floats
binding!(F64Binding, "F64Binding", f64, imp_f64);
binding!(F32Binding, "F32Binding", f32, imp_f32);

// String
binding!(StringBinding, "StringBinding", String, imp_string);
