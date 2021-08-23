use gtk::glib;
use gtk::prelude::{BoxExt, GridExt, TreeViewExt};

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

impl<Widget> FactoryView<Widget> for gtk::ListBox
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    fn add(&self, widget: &Widget, _position: &()) {
        self.append(widget);
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::ListBox
where
    Self: FactoryView<Widget>,
    Widget: gtk::prelude::ListBoxRowExt + glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) {
        self.insert(widget, other.index());
    }

    fn push_front(&self, widget: &Widget) {
        self.prepend(widget);
    }
}

impl FactoryView<gtk::TreeViewColumn> for gtk::TreeView {
    type Position = ();
    fn add(&self, widget: &gtk::TreeViewColumn, _position: &()) {
        self.insert_column(widget, -1);
    }

    fn remove(&self, widget: &gtk::TreeViewColumn) {
        self.remove_column(widget);
    }
}

impl<Widget> FactoryView<Widget> for gtk::FlowBox
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    fn add(&self, widget: &Widget, _position: &()) {
        self.insert(widget, -1);
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::FlowBox
where
    Self: FactoryView<Widget>,
    Widget: gtk::prelude::FlowBoxChildExt + glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) {
        self.insert(widget, other.index());
    }

    fn push_front(&self, widget: &Widget) {
        self.insert(widget, 0);
    }
}

#[derive(Debug)]
pub struct FixedPosition {
    pub x: f64,
    pub y: f64,
}

impl<Widget> FactoryView<Widget> for gtk::Fixed
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = FixedPosition;
    fn add(&self, widget: &Widget, position: &FixedPosition) {
        gtk::prelude::FixedExt::put(self, widget, position.x, position.y);
    }

    fn remove(&self, widget: &Widget) {
        gtk::prelude::FixedExt::remove(self, widget);
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

#[cfg(feature = "libadwaita")]
#[cfg_attr(doc, doc(cfg(feature = "libadwaita")))]
mod adwaita {
    use crate::factory::FactoryView;
    use gtk::glib;

    impl<Widget> FactoryView<Widget> for adw::Carousel
    where
        Widget: glib::IsA<gtk::Widget>,
    {
        type Position = ();
        fn add(&self, widget: &Widget, _position: &Self::Position) {
            self.append(widget);
        }

        fn remove(&self, widget: &Widget) {
            self.remove(widget);
        }
    }
}
