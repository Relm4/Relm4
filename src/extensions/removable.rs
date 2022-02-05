use crate::RelmSetChildExt;
use gtk::prelude::*;

/// Widget types which can have widgets removed from them.
pub trait RelmRemovableExt {
    /// Removes the widget from the container
    /// if it is a child of the container.
    fn container_remove(&self, widget: &impl AsRef<gtk::Widget>);
}

impl<T: RelmSetChildExt> RelmRemovableExt for T {
    fn container_remove(&self, _widget: &impl AsRef<gtk::Widget>) {
        self.container_set_child(None::<&gtk::Widget>);
    }
}

macro_rules! remove_impl {
    ($($type:ty),+) => {
        $(
            impl RelmRemovableExt for $type {
                fn container_remove(&self, widget: &impl AsRef<gtk::Widget>) {
                    self.remove(widget.as_ref());
                }
            }
        )+
    }
}

macro_rules! remove_child_impl {
    ($($type:ty),+) => {
        $(
            impl RelmRemovableExt for $type {
                fn container_remove(&self, widget: &impl AsRef<gtk::Widget>) {
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
    gtk::FlowBox,
    gtk::ListBox,
    gtk::Stack,
    gtk::HeaderBar
);
remove_child_impl!(gtk::InfoBar);
