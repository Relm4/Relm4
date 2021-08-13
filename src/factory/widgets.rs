/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use gtk::glib;
use gtk::prelude::{BoxExt, GridExt};

use crate::factory::FactoryView;

pub struct GridPosition {
    pub column: i32,
    pub row: i32,
    pub width: i32,
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
