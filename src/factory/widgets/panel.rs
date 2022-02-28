use crate::factory::{FactoryListView, FactoryView};
use gtk::glib;

impl<Widget> FactoryView<Widget> for panel::Paned
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    type Root = Widget;

    fn add(&self, widget: &Widget, _position: &Self::Position) -> Widget {
        self.append(widget);
        widget.clone()
    }

    fn remove(&self, widget: &Widget) {
        self.remove(widget);
    }
}

impl<Widget> FactoryListView<Widget> for panel::Paned
where
    Widget: glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &Widget) -> Widget {
        self.insert_after(widget, other);
        widget.clone()
    }

    fn push_front(&self, widget: &Widget) -> Widget {
        self.insert(0, widget);
        widget.clone()
    }
}
