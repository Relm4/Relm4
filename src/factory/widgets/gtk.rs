use gtk::prelude::{BoxExt, Cast, GridExt, ListBoxRowExt, WidgetExt};

use crate::factory::{positions, FactoryView, FactoryViewPlus};

impl FactoryView for gtk::Box {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;

    type Position = ();

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.append(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }
}

impl FactoryViewPlus for gtk::Box {
    fn factory_prepend(&self, widget: impl AsRef<Self::Children>) -> Self::ReturnedWidget {
        self.prepend(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
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

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
    }
}

impl FactoryView for gtk::Stack {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::StackPage;

    type Position = ();

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.add_child(widget.as_ref())
    }

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(&widget.child());
    }
}

impl FactoryView for gtk::ListBox {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::ListBoxRow;

    type Position = ();

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ReturnedWidget {
        self.append(widget.as_ref());
        widget.as_ref().parent().unwrap().downcast().unwrap()
    }

    fn factory_remove(&self, widget: &Self::ReturnedWidget) {
        self.remove(&widget.child().unwrap());
    }
}

impl FactoryViewPlus for gtk::ListBox {
    fn factory_prepend(&self, widget: impl AsRef<Self::Children>) -> Self::ReturnedWidget {
        self.prepend(widget.as_ref());
        widget.as_ref().parent().unwrap().downcast().unwrap()
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        other: &Self::ReturnedWidget,
    ) -> Self::ReturnedWidget {
        self.insert(widget.as_ref(), other.index());
        widget.as_ref().parent().unwrap().downcast().unwrap()
    }

    fn factory_move_after(&self, widget: &Self::ReturnedWidget, other: &Self::ReturnedWidget) {
        self.remove(widget);
        self.insert(widget, other.index());
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.remove(widget);
        self.prepend(widget);
    }

    fn returned_widget_to_child(returned_widget: &Self::ReturnedWidget) -> Self::Children {
        returned_widget.child().unwrap()
    }
}

