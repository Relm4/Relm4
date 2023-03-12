#![allow(missing_docs)]

mod bindings;
mod widgets;

pub use bindings::*;

use std::ops::{Deref, DerefMut};

use gtk::{glib, prelude::IsA};

pub trait ConnectBinding {
    type Target;

    fn bind<B: Binding<Target = Self::Target>>(&self, binding: &B);
}

pub trait ConnectBindingExt {
    type Target;
    fn with_binding<B: Binding<Target = Self::Target>>(value: &B) -> Self;
}

impl<T> ConnectBindingExt for T
where
    T: Default + ConnectBinding,
{
    type Target = <T as ConnectBinding>::Target;

    fn with_binding<B: Binding<Target = Self::Target>>(value: &B) -> Self {
        let obj = Self::default();
        obj.bind(value);
        obj
    }
}

#[derive(Debug)]
pub struct BindingGuard<B: Binding> {
    value: Option<B::Target>,
    inner: B,
}

impl<B: Binding> Deref for BindingGuard<B> {
    type Target = <B as Binding>::Target;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<B: Binding> DerefMut for BindingGuard<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<B: Binding> Drop for BindingGuard<B> {
    fn drop(&mut self) {
        self.inner.set(self.value.take().unwrap());
    }
}

pub trait Binding: Clone + IsA<glib::Object> {
    type Target;

    #[must_use]
    fn property_name() -> &'static str {
        "value"
    }

    fn guard(&self) -> BindingGuard<Self> {
        BindingGuard {
            value: Some(self.get()),
            inner: self.clone(),
        }
    }

    fn get(&self) -> Self::Target;
    fn set(&self, value: Self::Target);
}
