use crate::RelmSetChildExt;
use gtk::prelude::*;

/// Widget types which can have widgets removed from them.
pub trait RelmRemovableExt<'a> {
    /// Type of children of the container.
    type Child: 'a;
    /// Removes the widget from the container
    /// if it is a child of the container.
    fn container_remove(&self, widget: Self::Child);
    /// Remove all children from the container.
    fn remove_all(&self);
}

impl<'a, T: RelmSetChildExt> RelmRemovableExt<'a> for T {
    type Child = &'a dyn AsRef<gtk::Widget>;
    fn container_remove(&self, _widget: Self::Child) {
        self.container_set_child(None::<&gtk::Widget>);
    }
    fn remove_all(&self) {
        self.container_set_child(None::<&gtk::Widget>);
    }
}

impl<'a> RelmRemovableExt<'a> for gtk::ListBox {
    type Child = &'a gtk::ListBoxRow;
    fn container_remove(&self, widget: Self::Child) {
        self.remove(widget);
    }
    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            self.remove(&child);
        }
    }
}

macro_rules! remove_impl {
    ($($type:ty),+) => {
        $(
            impl<'a> RelmRemovableExt<'a> for $type {
                type Child = &'a dyn AsRef<gtk::Widget>;
                fn container_remove(&self, widget: Self::Child) {
                    self.remove(widget.as_ref());
                }
                fn remove_all(&self) {
                    while let Some(child) = self.last_child() {
                        self.remove(&child);
                    }
                }
            }
        )+
    }
}

macro_rules! remove_child_impl {
    ($($type:ty),+) => {
        $(
            impl<'a> RelmRemovableExt<'a> for $type {
                type Child = &'a dyn AsRef<gtk::Widget>;
                fn container_remove(&self, widget: Self::Child) {
                    self.remove_child(widget.as_ref());
                }
                fn remove_all(&self) {
                    while let Some(child) = self.last_child() {
                        self.remove_child(&child);
                    }
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
    gtk::Stack,
    gtk::HeaderBar
);
remove_child_impl!(gtk::InfoBar);
