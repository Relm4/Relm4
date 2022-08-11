use super::ContainerChild;
use crate::gtk;
use gtk::prelude::{Cast, GridExt, IsA, WidgetExt};

/// An iterator over container children.
#[derive(Debug)]
struct ChildrenIterator<T: RelmIterChildrenExt> {
    start: Option<T::Child>,
    end: Option<T::Child>,
    done: bool,
}

impl<T: RelmIterChildrenExt> ChildrenIterator<T> {
    /// Create a new iterator over children of `widget`.
    fn new(widget: &T) -> Self {
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
pub trait RelmIterChildrenExt: ContainerChild + IsA<gtk::Widget> {
    /// Returns an iterator over container children.
    fn iter_children(&self) -> Box<dyn DoubleEndedIterator<Item = Self::Child>> {
        Box::new(ChildrenIterator::new(self))
    }
}

impl RelmIterChildrenExt for gtk::Box {}
impl RelmIterChildrenExt for gtk::ListBox {}
impl RelmIterChildrenExt for gtk::FlowBox {}
impl RelmIterChildrenExt for gtk::Grid {
    // `gtk::Grid` places children in the order they were added to the grid.
    //
    // We have to provide a separate implementation that would sort children
    // depending on their position.
    fn iter_children(&self) -> Box<dyn DoubleEndedIterator<Item = Self::Child>> {
        let mut vec = Vec::new();
        let mut widget = self.first_child();
        while let Some(child) = widget {
            widget = child.next_sibling();
            let (column, row, _, _) = self.query_child(&child);
            vec.push((column, row, child));
        }

        vec.sort_by(|(col_a, row_a, _), (col_b, row_b, _)| {
            if row_a == row_b {
                col_a.cmp(col_b)
            } else {
                row_a.cmp(row_b)
            }
        });

        Box::new(vec.into_iter().map(|(_, _, child)| child))
    }
}
impl RelmIterChildrenExt for gtk::Stack {}

#[cfg(feature = "libadwaita")]
mod libadwaita {
    use super::RelmIterChildrenExt;
    use crate::gtk;
    use gtk::prelude::{Cast, ListModelExt};

    impl RelmIterChildrenExt for adw::TabView {
        fn iter_children(&self) -> Box<dyn DoubleEndedIterator<Item = Self::Child>> {
            let pages = self.pages();
            Box::new(
                (0..pages.n_items())
                    .filter_map(move |index| pages.item(index))
                    .filter_map(|item| item.downcast::<adw::TabPage>().ok())
                    .map(|page| page.child()),
            )
        }
    }
}
