//! Typed views.

pub mod column;
pub mod list;
mod selection_ext;

use self::selection_ext::RelmSelectionExt;
use gtk::{glib, prelude::Cast};
use std::{
    cell::{Ref, RefMut},
    cmp::Ordering,
    marker::PhantomData,
};

/// Sorting function used for views.
pub type OrdFn<T> = Option<Box<dyn Fn(&T, &T) -> Ordering>>;

struct Filter {
    filter: gtk::CustomFilter,
    model: gtk::FilterListModel,
}
/// And item of a [`list::TypedListView`].
///
/// The interface is very similar to [`std::cell::RefCell`].
/// Ownership is calculated at runtime, so you have to borrow the
/// value explicitly which might panic if done incorrectly.
#[derive(Debug, Clone)]
pub struct TypedListItem<T> {
    inner: glib::BoxedAnyObject,
    _ty: PhantomData<*const T>,
}

impl<T: 'static> TypedListItem<T> {
    fn new(inner: glib::BoxedAnyObject) -> Self {
        Self {
            inner,
            _ty: PhantomData,
        }
    }

    // rustdoc-stripper-ignore-next
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    ///
    /// For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[must_use]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    // rustdoc-stripper-ignore-next
    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

fn get_value<T: 'static>(obj: &glib::Object) -> Ref<'_, T> {
    let wrapper = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
    wrapper.borrow()
}

fn get_mut_value<T: 'static>(obj: &glib::Object) -> RefMut<'_, T> {
    let wrapper = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
    wrapper.borrow_mut()
}
