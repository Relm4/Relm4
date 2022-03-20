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
        self.reorder_page(widget, new_position);
    }

    fn factory_move_start(&self, widget: &Self::ReturnedWidget) {
        self.reorder_first(widget);
    }
}
