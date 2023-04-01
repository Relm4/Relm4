use std::{any::Any, mem};

use gtk::glib::Bytes;


pub(super) struct AnyWrapper;

impl AnyWrapper {
    pub(super) unsafe fn new<T: Any + 'static>(inner: T) -> (Bytes, Box<dyn Any>) {
        let value: Box<dyn Any> = Box::new(inner);
        let raw: *mut dyn Any = Box::into_raw(value);

        let bytes: [u8; 16] = unsafe { mem::transmute(raw) };
        let bytes = Bytes::from_owned(bytes);

        let dropper = Box::from_raw(raw);

        (bytes, dropper)
    }

    pub(super) unsafe fn from_bytes<T: Any>(bytes: Bytes) -> mem::ManuallyDrop<Box<T>> {
        let bytes = <[u8; 16]>::try_from(bytes.as_ref()).unwrap();
        let addr: *mut dyn Any = mem::transmute(bytes);
        mem::ManuallyDrop::new(Box::from_raw(addr).downcast().unwrap())
    }
}
