use crate::factory::{FactoryListView, FactoryView};
use gtk::glib;

impl<Widget> FactoryView<Widget> for adw::Carousel
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

impl<Widget> FactoryView<Widget> for adw::TabView
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    type Root = adw::TabPage;

    fn add(&self, widget: &Widget, _position: &()) -> adw::TabPage {
        self.append(widget).unwrap()
    }

    fn remove(&self, widget: &adw::TabPage) {
        self.close_page_finish(widget, true);
    }
}

impl<Widget> FactoryListView<Widget> for adw::TabView
where
    Widget: glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &adw::TabPage) -> adw::TabPage {
        let position = self.page_position(other) + 1;
        self.insert(widget, position).unwrap()
    }

    fn push_front(&self, widget: &Widget) -> adw::TabPage {
        self.prepend(widget).unwrap()
    }
}
