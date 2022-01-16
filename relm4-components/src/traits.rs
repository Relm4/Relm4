use relm4::gtk;

/// Get the parent window that allows setting the parent window of a dialog with
/// [`gtk::prelude::GtkWindowExt::set_transient_for`].
pub trait ParentWindow {
    /// Returns the parent window that a dialog should use or [`None`] if
    /// no parent window should be set
    fn parent_window(&self) -> Option<gtk::Window>;
}
