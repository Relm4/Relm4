mod container;
mod removable;
mod set_child;

#[cfg(test)]
mod tests;

#[allow(unreachable_pub)]
pub use self::container::RelmContainerExt;
#[allow(unreachable_pub)]
pub use self::removable::RelmRemovableExt;
#[allow(unreachable_pub)]
pub use self::set_child::RelmSetChildExt;

use gtk::prelude::*;

/// Additional methods for `gtk::builders::ApplicationBuilder`
pub trait ApplicationBuilderExt {
    /// Convenience method for launching an application and initializing the window.
    fn launch<F>(self, init: F)
    where
        F: Fn(gtk::Application, gtk::ApplicationWindow) + 'static;
}

impl ApplicationBuilderExt for gtk::builders::ApplicationBuilder {
    fn launch<F>(self, init: F)
    where
        F: Fn(gtk::Application, gtk::ApplicationWindow) + 'static,
    {
        let app = self.build();

        app.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::new(app);

            init(app.clone(), window.clone());

            window.show();
        });

        app.run();
    }
}

/// Additional methods for `gtk::Widget`
pub trait RelmWidgetExt {
    /// Attach widget to a `gtk::SizeGroup`.
    fn set_size_group(&self, size_group: &gtk::SizeGroup);

    /// Locate the top level window this widget is attached to.
    ///
    /// Equivalent to `widget.ancestor(gtk::Window::static_type())`, then casting.
    fn toplevel_window(&self) -> Option<gtk::Window>;
}

impl<T: gtk::glib::IsA<gtk::Widget>> RelmWidgetExt for T {
    fn set_size_group(&self, size_group: &gtk::SizeGroup) {
        size_group.add_widget(self);
    }

    fn toplevel_window(&self) -> Option<gtk::Window> {
        self.ancestor(gtk::Window::static_type())
            .and_then(|widget| widget.dynamic_cast::<gtk::Window>().ok())
    }
}

fn iter_children(widget: &gtk::Widget) -> impl Iterator<Item = gtk::Widget> {
    let mut widget = widget.first_child();

    std::iter::from_fn(move || {
        if let Some(child) = widget.take() {
            widget = child.next_sibling();
            return Some(child);
        }

        None
    })
}

/// Additional methods for `gtk::Box`.
pub trait RelmBoxExt {
    /// Returns all children of the box.
    fn children(&self) -> Vec<gtk::Widget>;

    /// Remove all children from the box.
    fn remove_all(&self);
}

impl RelmBoxExt for gtk::Box {
    fn children(&self) -> Vec<gtk::Widget> {
        iter_children(self.as_ref()).collect()
    }
    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            self.remove(&child);
        }
    }
}

/// Additional methods for `gtk::ListBox`.
pub trait RelmListBoxExt {
    /// Returns all rows of the listbox.
    fn rows(&self) -> Vec<gtk::ListBoxRow>;

    /// Get the index of a widget attached to a listbox.
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32>;

    /// Remove all children from listbox.
    fn remove_all(&self);

    /// Remove the row of a child attached a listbox.
    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>);

    /// Get the row of a widget attached to a listbox.
    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow>;
}

impl RelmListBoxExt for gtk::ListBox {
    fn rows(&self) -> Vec<gtk::ListBoxRow> {
        iter_children(self.as_ref())
            .map(|widget| {
                widget
                    .downcast::<gtk::ListBoxRow>()
                    .expect("The child of `ListBox` is not a `ListBoxRow`.")
            })
            .collect()
    }
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32> {
        if let Some(row) = self.row_of_child(widget) {
            return Some(row.index());
        }

        None
    }

    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            let row = child
                .downcast::<gtk::ListBoxRow>()
                .expect("The child of `ListBox` is not a `ListBoxRow`.");
            row.set_child(None::<&gtk::Widget>);
            self.remove(&row);
        }
    }

    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>) {
        if let Some(row) = self.row_of_child(widget) {
            row.set_child(None::<&gtk::Widget>);
            self.remove(&row);
        }
    }

    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow> {
        if let Some(row) = widget.as_ref().ancestor(gtk::ListBoxRow::static_type()) {
            if let Some(row) = row.downcast_ref::<gtk::ListBoxRow>() {
                if let Some(parent_widget) = row.parent() {
                    if let Some(parent_box) = parent_widget.downcast_ref::<gtk::ListBox>() {
                        if parent_box == self {
                            return Some(row.clone());
                        }
                    }
                }
            }
        }

        None
    }
}

/// Additional methods for `gtk::FlowBox`.
pub trait RelmFlowBoxExt {
    /// Returns all children of the flowbox.
    fn flow_children(&self) -> Vec<gtk::FlowBoxChild>;

    /// Remove all children from the flowbox.
    fn remove_all(&self);
}

impl RelmFlowBoxExt for gtk::FlowBox {
    fn flow_children(&self) -> Vec<gtk::FlowBoxChild> {
        iter_children(self.as_ref())
            .map(|widget| {
                widget
                    .downcast::<gtk::FlowBoxChild>()
                    .expect("The child of `FlowBox` is not a `FlowBoxChild`.")
            })
            .collect()
    }
    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            self.remove(&child);
        }
    }
}

/// Additional methods for `gtk::Grid`.
pub trait RelmGridExt {
    /// Returns all children of the grid.
    fn children(&self) -> Vec<gtk::Widget>;

    /// Remove all children from the grid.
    fn remove_all(&self);
}

impl RelmGridExt for gtk::Grid {
    fn children(&self) -> Vec<gtk::Widget> {
        iter_children(self.as_ref()).collect()
    }
    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            self.remove(&child);
        }
    }
}
