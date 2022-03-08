use crate::RelmSetChildExt;
use gtk::prelude::*;

/// Widget types which can have widgets removed from them.
pub trait RelmRemovableExt<W> {
    /// Removes the widget from the container
    /// if it is a child of the container.
    fn container_remove(&self, widget: &W);
}

impl<T: RelmSetChildExt, W: AsRef<gtk::Widget>> RelmRemovableExt<W> for T {
    fn container_remove(&self, _widget: &W) {
        self.container_set_child(None::<&gtk::Widget>);
    }
}

impl<W: AsRef<gtk::ListBoxRow>> RelmRemovableExt<W> for gtk::ListBox {
    fn container_remove(&self, widget: &W) {
        let row = widget.as_ref();
        row.set_child(None::<&gtk::Widget>);
        self.remove(row);
    }
}

impl<W: AsRef<gtk::FlowBoxChild>> RelmRemovableExt<W> for gtk::FlowBox {
    fn container_remove(&self, widget: &W) {
        self.remove(widget.as_ref());
    }
}

macro_rules! remove_impl {
    ($($type:ty),+) => {
        $(
            impl<W: AsRef<gtk::Widget>> RelmRemovableExt<W> for $type {
                fn container_remove(&self, widget: &W) {
                    self.remove(widget.as_ref());
                }
            }
        )+
    }
}

macro_rules! remove_child_impl {
    ($($type:ty),+) => {
        $(
            impl<W: AsRef<gtk::Widget>> RelmRemovableExt<W> for $type {
                fn container_remove(&self, widget: &W) {
                    self.remove_child(widget.as_ref());
                }
            }
        )+
    }
}

remove_impl!(
    gtk::Box,
    gtk::Fixed,
    gtk::Grid,
    gtk::ActionBar,
    gtk::Stack,
    gtk::HeaderBar
);
remove_child_impl!(gtk::InfoBar);
