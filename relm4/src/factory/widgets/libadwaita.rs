#![allow(deprecated)]
use adw::prelude::*;

use crate::factory::FactoryView;

impl FactoryView for adw::TabView {
    type Children = gtk::Widget;
    type ReturnedWidget = adw::TabPage;

    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.close_page(widget);
        self.close_page_finish(widget, true);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.append(widget.as_ref())
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        self.prepend(widget.as_ref())
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        let new_position = self.page_position(other) + 1;
        self.insert(widget.as_ref(), new_position)
    }

    fn returned_widget_to_child(root_child: &Self::ReturnedWidget) -> Self::Children {
        root_child.child()
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        let new_position = self.page_position(other) + 1;
        if new_position == self.n_pages() {
            self.reorder_last(widget);
        } else {
            self.reorder_page(widget, new_position);
        }
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.reorder_first(widget);
    }
}

impl FactoryView for adw::PreferencesPage {
    type Children = adw::PreferencesGroup;
    type ReturnedWidget = adw::PreferencesGroup;

    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.add(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &(),
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &(),
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn returned_widget_to_child(root_child: &Self::ReturnedWidget) -> Self::Children {
        root_child.clone()
    }

    fn factory_move_after(&self, _widget: &Self::ReturnedWidget, _other: &Self::ReturnedWidget) {}

    fn factory_move_start(&self, _widget: &Self::ReturnedWidget) {}
}

impl FactoryView for adw::ExpanderRow {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.add_row(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget, position)
    }

    fn factory_move_after(&self, _widget: &Self::ReturnedWidget, _other: &Self::ReturnedWidget) {}

    fn factory_move_start(&self, _widget: &Self::ReturnedWidget) {}

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.clone()
    }

    fn factory_update_position(&self, widget: &Self::ReturnedWidget, position: &Self::Position) {
        self.factory_remove(widget);
        self.factory_append(widget, position);
    }
}

impl FactoryView for adw::Carousel {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.append(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        self.prepend(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &(),
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.factory_append(widget.as_ref(), position);
        widget.as_ref().clone()
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.clone()
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        for i in 0..self.n_pages() {
            if self.nth_page(i).eq(other) {
                self.reorder(widget, (i + 1) as i32);
                return;
            }
        }
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.reorder(widget, 0);
    }
}

impl FactoryView for adw::PreferencesGroup {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.add(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        self.add(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.add(widget.as_ref());
        widget.as_ref().clone()
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.clone()
    }

    fn factory_move_after(&self, _widget: &Self::ReturnedWidget, _other: &Self::ReturnedWidget) {}

    fn factory_move_start(&self, _widget: &Self::ReturnedWidget) {}
}

impl FactoryView for adw::Leaflet {
    type Children = gtk::Widget;
    type ReturnedWidget = adw::LeafletPage;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(&widget.child());
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.append(widget.as_ref())
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        self.prepend(widget.as_ref())
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.insert_child_after(widget.as_ref(), Some(&other.child()))
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        self.reorder_child_after(&widget.child(), Some(&other.child()))
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.reorder_child_after(&widget.child(), None::<&gtk::Widget>);
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.child()
    }
}
