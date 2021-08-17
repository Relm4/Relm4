use gtk::glib;
use gtk::prelude::{BoxExt, GridExt};

use crate::factory::{FactoryListView, FactoryView};

/// Storing information about where new widgets can be placed
/// inside a [`gtk::Grid`].
#[derive(Debug)]
pub struct GridPosition {
    /// The number of the column.
    pub column: i32,
    /// The number of the row.
    pub row: i32,
    /// The amount of columns the widget should take.
    pub width: i32,
    /// The amount of rows the widget should take.
    pub height: i32,
}

impl<Widget> FactoryView<Widget> for gtk::Box
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    fn add(&self, widget: &Widget, _position: &()) {
        self.append(widget);
    }

    fn remove(&self, widget: &Widget) {
        BoxExt::remove(self, widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::Box
where
    Self: FactoryView<Widget>,
    Widget: glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) {
        self.insert_child_after(widget, Some(other));
    }

    fn push_front(&self, widget: &Widget) {
        self.prepend(widget);
    }
}

impl<Widget> FactoryView<Widget> for gtk::Grid
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = GridPosition;

    fn add(&self, widget: &Widget, position: &GridPosition) {
        self.attach(
            widget,
            position.column,
            position.row,
            position.width,
            position.height,
        );
    }

    fn remove(&self, widget: &Widget) {
        GridExt::remove(self, widget);
    }
}
