use crate::factory::{
    positions::{StackPageInfo, TabPageInfo},
    FactoryListView, FactoryView,
};
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
    type Position = TabPageInfo;
    type Root = adw::TabPage;

    fn add(&self, widget: &Widget, position: &TabPageInfo) -> adw::TabPage {
        let page = self.append(widget);

        if let Some(title) = &position.title {
            page.set_title(title);
        }

        if let Some(tooltip) = &position.tooltip {
            page.set_tooltip(tooltip);
        }

        page
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
        self.insert(widget, position)
    }

    fn push_front(&self, widget: &Widget) -> adw::TabPage {
        self.prepend(widget)
    }
}

impl<Widget> FactoryView<Widget> for adw::ViewStack
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

impl<Widget> FactoryView<Widget> for adw::Leaflet
where
    Widget: glib::IsA<gtk::Widget>,
{
    type Position = ();
    type Root = adw::LeafletPage;

    fn add(&self, widget: &Widget, _position: &()) -> adw::LeafletPage {
        self.append(widget)
    }

    fn remove(&self, widget: &adw::LeafletPage) {
        self.remove(&widget.child());
    }
}

impl<Widget> FactoryListView<Widget> for adw::Leaflet
where
    Widget: glib::IsA<gtk::Widget>,
{
    fn insert_after(&self, widget: &Widget, other: &adw::LeafletPage) -> adw::LeafletPage {
        self.insert_child_after(widget, Some(&other.child()))
    }

    fn push_front(&self, widget: &Widget) -> adw::LeafletPage {
        self.prepend(widget)
    }
}
