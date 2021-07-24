use gtk::glib;
use gtk::prelude::{BoxExt, GridExt};

use crate::generator::GeneratorWidget;

impl<Widget> GeneratorWidget<Widget, ()> for gtk::Box
where
    Widget: glib::IsA<gtk::Widget>,
{
    fn add(&self, widget: &Widget, _position: &()) {
        self.append(widget);
    }

    fn remove(&self, widget: &Widget) {
        BoxExt::remove(self, widget);
    }
}

pub struct GridPosition {
    pub column: i32,
    pub row: i32,
    pub width: i32,
    pub height: i32,
}

impl<Widget> GeneratorWidget<Widget, GridPosition> for gtk::Grid
where
    Widget: glib::IsA<gtk::Widget>,
{
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
