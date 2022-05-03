use crate::factory::FactoryView;

impl FactoryView for panel::Paned {
    type Children = gtk::Widget;
    type ReturnedWidget = gtk::Widget;
    type Position = ();

    fn factory_remove(&self, widget: &gtk::Widget) {
        self.remove(widget);
    }

    fn factory_append(&self, widget: impl AsRef<gtk::Widget>, _: &()) -> gtk::Widget {
        self.append(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_prepend(&self, widget: impl AsRef<gtk::Widget>, _: &()) -> gtk::Widget {
        self.prepend(widget.as_ref());
        widget.as_ref().clone()
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<gtk::Widget>,
        _: &(),
        other: &gtk::Widget,
    ) -> gtk::Widget {
        self.insert_after(widget.as_ref(), other);
        widget.as_ref().clone()
    }

    fn returned_widget_to_child(root_child: &gtk::Widget) -> gtk::Widget {
        root_child.clone()
    }

    fn factory_move_after(&self, widget: &gtk::Widget, other: &gtk::Widget) {
        self.insert_after(widget, other);
    }

    fn factory_move_start(&self, widget: &gtk::Widget) {
        self.insert(0, widget);
    }
}
