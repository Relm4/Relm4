use gtk::glib;
use gtk::prelude::{BoxExt, GridExt, TreeViewExt};

use crate::factory::{positions::*, FactoryListView, FactoryView};

impl<Widget> FactoryView<Widget> for gtk::Box
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    type Root = Widget;
    fn add(&self, widget: &Widget, _position: &()) -> Widget {
        self.append(widget);
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        BoxExt::remove(self, widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::Box
where
    Self: FactoryView<Widget, Root = Widget>,
    Widget: glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) -> Widget {
        self.insert_child_after(widget, Some(other));
        widget.clone()
    }

    fn push_front(&self, widget: &Widget) -> Widget{
        self.prepend(widget);
        widget.clone()
    }
}

impl<Widget> FactoryView<Widget> for gtk::ListBox
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    type Root = Widget;

    fn add(&self, widget: &Widget, _position: &()) -> Widget {
        self.append(widget);
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::ListBox
where
    Self: FactoryView<Widget, Root = Widget>,
    Widget: gtk::prelude::ListBoxRowExt + glib::IsA<gtk::Widget> + Clone,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) -> Widget {
        self.insert(widget, other.index());
        widget.clone()
    }

    fn push_front(&self, widget: &Widget) -> Widget {
        self.prepend(widget);
        widget.clone()
    }
}

impl FactoryView<gtk::TreeViewColumn> for gtk::TreeView {
    type Position = ();
    type Root = gtk::TreeViewColumn;

    fn add(&self, widget: &gtk::TreeViewColumn, _position: &()) -> gtk::TreeViewColumn {
        self.insert_column(widget, -1);
        widget.clone()
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
    type Root = Widget;

    fn add(&self, widget: &Widget, _position: &()) -> Widget {
        self.insert(widget, -1);
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryListView<Widget> for gtk::FlowBox
where
    Self: FactoryView<Widget, Root = Widget>,
    Widget: gtk::prelude::FlowBoxChildExt + glib::IsA<gtk::Widget> + Clone,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) -> Widget {
        self.insert(widget, other.index());
        widget.clone()
    }

    fn push_front(&self, widget: &Widget) -> Widget {
        self.insert(widget, 0);
        widget.clone()
    }
}

impl<Widget> FactoryView<Widget> for gtk::Stack
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = StackPageInfo;
    type Root = Widget;

    fn add(&self, widget: &Widget, position: &StackPageInfo) -> Widget {
        if let Some(title) = &position.title {
            self.add_titled(widget, position.name.as_deref(), title);
        } else {
            self.add_named(widget, position.name.as_deref());
        }
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryView<Widget> for gtk::Fixed
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = FixedPosition;
    type Root = Widget;

    fn add(&self, widget: &Widget, position: &FixedPosition) -> Widget {
        gtk::prelude::FixedExt::put(self, widget, position.x, position.y);
        widget.clone()
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
    type Root = Widget;

    fn add(&self, widget: &Widget, position: &GridPosition) -> Widget {
        self.attach(
            widget,
            position.column,
            position.row,
            position.width,
            position.height,
        );
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        GridExt::remove(self, widget);
    }
}
