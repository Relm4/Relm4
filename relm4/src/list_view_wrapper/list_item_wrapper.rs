use std::{any::Any, mem};

use gtk::{glib, prelude::{Cast, ObjectExt}};

use super::any_wrapper::AnyWrapper;

pub(super) fn get_value_from_wrapper<T: 'static>(obj: &glib::Object) -> mem::ManuallyDrop<Box<T>> {
    let wrapper = obj.downcast_ref::<ListItemWrapper>().unwrap();
    wrapper.get()
}

glib::wrapper! {
    pub(super) struct ListItemWrapper(ObjectSubclass<imp::ListItemWrapper>);
}

impl ListItemWrapper {
    pub(super) fn new<T: Any + 'static>(value: T) -> Self {
        let this: Self = glib::Object::new();
        let (bytes, dropper) = unsafe { AnyWrapper::new(value) };
        this.set_value(bytes);

        this.add_weak_ref_notify_local(move || drop(dropper));

        this
    }

    pub(super) fn get<T: Any>(&self) -> mem::ManuallyDrop<Box<T>> {
        let bytes = self.value().unwrap();
        unsafe { AnyWrapper::from_bytes(bytes) }
    }
}

#[allow(missing_docs, unreachable_pub)]
mod imp {
    use std::cell::RefCell;

    use glib::prelude::*;
    use glib::{ParamSpec, Properties, Value};
    use gtk::glib::Bytes;
    use gtk::subclass::prelude::ObjectImpl;
    use gtk::{
        glib,
        subclass::prelude::{DerivedObjectProperties, ObjectSubclass},
    };

    #[derive(Default, Properties, Debug)]
    #[properties(wrapper_type = super::ListItemWrapper)]
    /// Inner type of the data binding.
    pub(in crate::list_view_wrapper) struct ListItemWrapper {
        #[property(get, set)]
        /// The primary value.
        value: RefCell<Option<Bytes>>,
    }

    impl ObjectImpl for ListItemWrapper {
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
    impl ObjectSubclass for ListItemWrapper {
        const NAME: &'static str = "ListItemWrapper";
        type Type = super::ListItemWrapper;
    }
}
