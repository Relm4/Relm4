/// Get the parent window that allows setting the parent window of a dialog with
/// [`gtk::prelude::GtkWindowExt::set_transient_for`].
pub trait ParentWindow {
    fn parent_window(&self) -> Option<gtk::Window>;
}
