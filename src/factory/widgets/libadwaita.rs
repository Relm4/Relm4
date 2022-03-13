impl FactoryView for adw::TabView {
    type Children = gtk::Widget;
    type ChildRoot = adw::TabPage;

    type Position = ();

    fn factory_append(
        &self,
        widget: impl AsRef<Self::Children>,
        _position: &Self::Position,
    ) -> Self::ChildRoot {
        self.append(widget.as_ref())
    }

    fn factory_remove(&self, widget: &Self::ChildRoot) {
        self.close_page_finish(widget, true);
    }
}

impl FactoryViewPlus for adw::TabView {
    fn factory_prepend(&self, widget: impl AsRef<Self::Children>) -> Self::ChildRoot {
        self.prepend(widget.as_ref())
    }

    fn factory_insert_after(
        &self,
        widget: impl AsRef<Self::Children>,
        other: &Self::ChildRoot,
    ) -> Self::ChildRoot {
        let new_position = self.page_position(other) + 1;
        self.insert(widget.as_ref(), new_position)
    }

    fn child_root_to_child(root_child: &Self::ChildRoot) -> Self::Children {
        root_child.child()
    }

    fn factory_move_after(&self, widget: &Self::ChildRoot, other: &Self::ChildRoot) {
        let new_position = self.page_position(other) + 1;
        self.reorder_page(widget, new_position);
    }

    fn factory_move_start(&self, widget: &Self::ChildRoot) {
        self.reorder_first(widget);
    }
}
