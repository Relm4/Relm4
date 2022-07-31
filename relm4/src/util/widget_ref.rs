use gtk::Widget;

/// Get a reference to a widget.
///
/// This trait is an extension of [`AsRef`]
/// that always returns `&`[`Widget`].
pub trait WidgetRef {
    /// Returns a reference to a widget.
    ///
    /// Like [`AsRef::as_ref`] it will auto-dereference.
    fn widget_ref(&self) -> &Widget;
}

impl<T: AsRef<Widget>> WidgetRef for T {
    fn widget_ref(&self) -> &Widget {
        self.as_ref()
    }
}
