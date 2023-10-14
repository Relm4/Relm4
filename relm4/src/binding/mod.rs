//! Easy data bindings between objects.
//!
//! Particularly, this module provides types and traits
//! that simplify connecting with the primary property of an object.
//! In this context, *primary* means that the property is the most important and
//! most accessed value stored by the object.
//! Usually, this is very easy to figure out, such as the *active* property for [`gtk::ToggleButton`].
//! In any case, the primary property type are always documented.
//!
//! To find out which widgets are currently supported and which property is considered as primary,
//! please look at the list implementers for [`ConnectBinding`].
//! Contributions to add support for more widgets are always welcome.

mod bindings;
mod widgets;

pub use bindings::*;

use std::ops::{Deref, DerefMut};

use gtk::{glib, prelude::IsA};

/// A trait that allows type-safe bindings between to the primary properties of two objects.
pub trait ConnectBinding {
    /// The type of the primary property.
    type Target;

    /// Create a type-safe bidirectional between the primary property of an object
    /// and a [`Binding`].
    fn bind<B: Binding<Target = Self::Target>>(&self, binding: &B);
}

/// Extension for [`ConnectBinding`].
/// This trait is not implemented manually, but through
/// automatically implemented for all types that implement
/// [`ConnectBinding`].
pub trait ConnectBindingExt {
    /// The type of the primary property.
    /// Inherited from [`ConnectBinding::Target`].
    type Target;

    /// Create an object and immediately connect it with a [`Binding`].
    fn with_binding<B: Binding<Target = Self::Target>>(binding: &B) -> Self;
}

impl<T> ConnectBindingExt for T
where
    T: Default + ConnectBinding,
{
    type Target = <T as ConnectBinding>::Target;

    fn with_binding<B: Binding<Target = Self::Target>>(binding: &B) -> Self {
        let obj = Self::default();
        obj.bind(binding);
        obj
    }
}

#[derive(Debug)]
/// A RAII-guard that stores a value to
/// a [`Binding`].
///
/// Once dropped, it will automatically update
/// the value of the primary property.
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

/// A [`glib::Object`] with one primary property.
pub trait Binding: Clone + IsA<glib::Object> {
    /// The type of the primary property.
    type Target;

    #[must_use]
    /// The name of the primary property.
    fn property_name() -> &'static str {
        "value"
    }

    /// Get a new [`BindingGuard`] from the object.
    ///
    /// Once dropped, the [`BindingGuard`] will
    /// automatically update the value of the
    /// primary property.
    fn guard(&self) -> BindingGuard<Self> {
        BindingGuard {
            value: Some(self.get()),
            inner: self.clone(),
        }
    }

    /// Get the value of the primary property.
    fn get(&self) -> Self::Target;

    /// Set the value of the primary property.
    fn set(&self, value: Self::Target);
}
