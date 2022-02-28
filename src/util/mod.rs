//! Utility traits for working with GTK widgets.

pub mod widget_plus;

/// Get a reference to a widget.
pub trait WidgetRef {
    /// Returns a reference to a widget.
    fn widget_ref(&self) -> &Widget;
}

use gtk::Widget;

impl<T: AsRef<Widget>> WidgetRef for T {
    fn widget_ref(&self) -> &Widget {
        self.as_ref()
    }
}    
