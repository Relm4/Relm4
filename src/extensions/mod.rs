mod container;
mod removable;
mod set_child;

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
    /// Iterates across the child widgets of a widget.
    fn iter_children(&self) -> Box<dyn Iterator<Item = gtk::Widget>>;

    /// Iterates across the child widgets of a widget, in reverse order.
    fn iter_children_reverse(&self) -> Box<dyn Iterator<Item = gtk::Widget>>;

    /// Iterates children of a widget with a closure.
    fn for_each_child<F: FnMut(gtk::Widget) + 'static>(&self, func: F);

    /// Attach widget to a `gtk::SizeGroup`.
    fn set_size_group(&self, size_group: &gtk::SizeGroup);

    /// Locate the top level window this widget is attached to.
    ///
    /// Equivalent to `widget.ancestor(gtk::Window::static_type())`, then casting.
    fn toplevel_window(&self) -> Option<gtk::Window>;
}

impl<T: gtk::glib::IsA<gtk::Widget>> RelmWidgetExt for T {
    fn for_each_child<F: FnMut(gtk::Widget) + 'static>(&self, mut func: F) {
        let mut widget = self.first_child();

        while let Some(child) = widget.take() {
            widget = child.next_sibling();
            func(child);
        }
    }

    fn iter_children(&self) -> Box<dyn Iterator<Item = gtk::Widget>> {
        Box::new(iter_children(self.as_ref()))
    }

    fn iter_children_reverse(&self) -> Box<dyn Iterator<Item = gtk::Widget>> {
        Box::new(iter_children_reverse(self.as_ref()))
    }

    fn set_size_group(&self, size_group: &gtk::SizeGroup) {
        size_group.add_widget(self);
    }

    fn toplevel_window(&self) -> Option<gtk::Window> {
        self.ancestor(gtk::Window::static_type())
            .and_then(|widget| widget.dynamic_cast::<gtk::Window>().ok())
    }
}

/// Additional methods for `gtk::ListBox`.
pub trait RelmListBoxExt {
    /// Get the index of a widget attached to a listbox.
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32>;

    /// Remove the row of a child attached a listbox.
    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>);

    /// Get the row of a widget attached to a listbox.
    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow>;
}

impl RelmListBoxExt for gtk::ListBox {
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32> {
        if let Some(row) = self.row_of_child(widget) {
            return Some(row.index());
        }

        None
    }

    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>) {
        if let Some(row) = self.row_of_child(widget) {
            self.remove(&row);
        }
    }

    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow> {
        if let Some(row) = widget.as_ref().ancestor(gtk::ListBoxRow::static_type()) {
            if let Some(row) = row.dynamic_cast_ref::<gtk::ListBoxRow>() {
                if let Some(parent_widget) = row.parent() {
                    if let Some(parent_box) = parent_widget.dynamic_cast_ref::<gtk::ListBox>() {
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

fn iter_children_reverse(widget: &gtk::Widget) -> impl Iterator<Item = gtk::Widget> {
    let mut widget = widget.last_child();

    std::iter::from_fn(move || {
        if let Some(child) = widget.take() {
            widget = child.prev_sibling();
            return Some(child);
        }

        None
    })
}
