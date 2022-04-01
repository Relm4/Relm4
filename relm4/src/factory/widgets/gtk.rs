use gtk::prelude::{BoxExt, Cast, FlowBoxChildExt, GridExt, ListBoxRowExt, WidgetExt};

use crate::factory::{positions, FactoryView};

impl FactoryView for gtk::Box {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
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
        _position: &(),
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.insert_child_after(widget.as_ref(), Some(other));
        widget.as_ref().clone()
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.clone()
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        self.reorder_child_after(widget, Some(other));
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.reorder_child_after(widget, None::<&gtk::Widget>);
    }
}

impl FactoryView for gtk::Grid {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = positions::GridPosition;

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.attach(
            widget.as_ref(),
            position.column,
            position.row,
            position.width,
            position.height,
        );
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

impl FactoryView for gtk::Stack {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::StackPage;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(&widget.child());
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.add_child(widget.as_ref())
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        self.add_child(widget.as_ref())
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        _other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.add_child(widget.as_ref())
    }

    fn factory_move_after(&self, _widget: &Self::ReturnedWidget, _other: &Self::ReturnedWidget) {}

    fn factory_move_start(&self, _widget: &Self::ReturnedWidget) {}

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.child()
    }
}

impl FactoryView for gtk::ListBox {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::ListBoxRow;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        widget.set_child(None::<&gtk::Widget>);
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.append(widget);

        match widget.downcast_ref::<gtk::ListBoxRow>() {
            Some(row) => row.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.prepend(widget);

        match widget.downcast_ref::<gtk::ListBoxRow>() {
            Some(row) => row.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.insert(widget, other.index() + 1);

        match widget.downcast_ref::<gtk::ListBoxRow>() {
            Some(row) => row.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        self.remove(widget);
        self.insert(widget, other.index() + 1);
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
        self.prepend(widget);
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.child().unwrap()
    }
}

impl FactoryView for gtk::FlowBox {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::FlowBoxChild;
    type Position = ();

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        widget.set_child(None::<&gtk::Widget>);
        self.remove(widget);
    }

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.insert(widget, -1);

        match widget.downcast_ref::<gtk::FlowBoxChild>() {
            Some(child) => child.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_prepend(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.insert(widget, 0);

        match widget.downcast_ref::<gtk::FlowBoxChild>() {
            Some(child) => child.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &(),
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        let widget = widget.as_ref();

        self.insert(widget, other.index() + 1);

        match widget.downcast_ref::<gtk::FlowBoxChild>() {
            Some(child) => child.clone(),
            None => widget.parent().unwrap().downcast().unwrap(),
        }
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        self.remove(widget);
        self.insert(widget, other.index() + 1);
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
        self.insert(widget, 0);
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.child().unwrap()
    }
}

// impl FactoryView<gtk::TreeViewColumn> for gtk::TreeView {
//     type Position = ();
//     type Root = gtk::TreeViewColumn;

//     fn add(&self, widget: &gtk::TreeViewColumn, _position: &()) -> gtk::TreeViewColumn {
//         self.insert_column(widget, -1);
//         widget.clone()
//     }

//     fn remove(&self, widget: &gtk::TreeViewColumn) {
//         self.remove_column(widget);
//     }
// }

// impl<Widget> FactoryView<Widget> for gtk::Stack
// where
//     Widget: glib::IsA<gtk::Widget>,
// {
//     type Position = StackPageInfo;
//     type Root = Widget;

//     fn add(&self, widget: &Widget, position: &StackPageInfo) -> Widget {
//         if let Some(title) = &position.title {
//             self.add_titled(widget, position.name.as_deref(), title);
//         } else {
//             self.add_named(widget, position.name.as_deref());
//         }
//         widget.clone()
//     }

//     fn remove(&self, widget: &Widget) {
//         self.remove(widget);
//     }
// }

// impl<Widget> FactoryView<Widget> for gtk::Fixed
// where
//     Widget: glib::IsA<gtk::Widget>,
// {
//     type Position = FixedPosition;
//     type Root = Widget;

//     fn add(&self, widget: &Widget, position: &FixedPosition) -> Widget {
//         gtk::prelude::FixedExt::put(self, widget, position.x, position.y);
//         widget.clone()
//     }

//     fn remove(&self, widget: &Widget) {
//         gtk::prelude::FixedExt::remove(self, widget);
//     }
// }

// impl<Widget> FactoryView<Widget> for gtk::Grid
// where
//     Widget: glib::IsA<gtk::Widget>,
// {
//     type Position = GridPosition;
//     type Root = Widget;

//     fn add(&self, widget: &Widget, position: &GridPosition) -> Widget {
//         self.attach(
//             widget,
//             position.column,
//             position.row,
//             position.width,
//             position.height,
//         );
//         widget.clone()
//     }

//     fn remove(&self, widget: &Widget) {
//         GridExt::remove(self, widget);
//     }
// }
