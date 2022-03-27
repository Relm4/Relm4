mod container;
mod remove;
mod set_child;

#[cfg(test)]
mod tests;

#[allow(unreachable_pub)]
pub use self::container::RelmContainerExt;
#[allow(unreachable_pub)]
pub use self::remove::{RelmRemoveAllExt, RelmRemoveExt};
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
        self.row_of_child(widget).map(|row| row.index())
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

/// An iterator over container children.
#[derive(Debug)]
pub struct ChildrenIterator<T: RelmIterChildrenExt> {
    start: Option<T::Child>,
    end: Option<T::Child>,
    done: bool,
}

impl<T: RelmIterChildrenExt> ChildrenIterator<T> {
    /// Create a new iterator over children of `widget`.
    pub fn new(widget: &T) -> Self {
        let start = widget.first_child().map(|child| {
            child
                .downcast::<T::Child>()
                .expect("The type of children does not match.")
        });
        let end = widget.last_child().map(|child| {
            child
                .downcast::<T::Child>()
                .expect("The type of children does not match.")
        });
        let done = start.is_none();
        ChildrenIterator { start, end, done }
    }
}

impl<T: RelmIterChildrenExt> Iterator for ChildrenIterator<T> {
    type Item = T::Child;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            // Handle cases where only one child exists and
            // when all but one widget were consumed
            if self.start == self.end {
                self.done = true;
                self.start.clone()
            } else if let Some(start) = self.start.take() {
                // "Increment" the start child
                self.start = start.next_sibling().map(|child| {
                    child
                        .downcast::<T::Child>()
                        .expect("The type of children does not match.")
                });
                // Just to make sure the iterator ends next time
                // because all widgets were consumed
                self.done = self.start.is_none();
                Some(start)
            } else {
                None
            }
        }
    }
}

impl<T: RelmIterChildrenExt> DoubleEndedIterator for ChildrenIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            // Handle cases where only one child exists and
            // when all but one widget were consumed
            if self.start == self.end {
                self.done = true;
                self.end.clone()
            } else if let Some(end) = self.end.take() {
                // "Decrement" the end child
                self.end = end.prev_sibling().map(|child| {
                    child
                        .downcast::<T::Child>()
                        .expect("The type of children does not match.")
                });
                // Just to make sure the iterator ends next time
                // because all widgets were consumed
                self.done = self.end.is_none();
                Some(end)
            } else {
                None
            }
        }
    }
}

/// Widget types which allow iteration over their children.
pub trait RelmIterChildrenExt: RelmRemoveExt + IsA<gtk::Widget> {
    /// Returns an iterator over container children.
    fn iter_children(&self) -> ChildrenIterator<Self> {
        ChildrenIterator::new(self)
    }
}

impl RelmIterChildrenExt for gtk::Box {}
impl RelmIterChildrenExt for gtk::ListBox {}
impl RelmIterChildrenExt for gtk::FlowBox {}
impl RelmIterChildrenExt for gtk::Grid {}
impl RelmIterChildrenExt for gtk::Stack {}
