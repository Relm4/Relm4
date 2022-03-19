use gtk::prelude::*;

use crate::RelmSetChildExt;

/// Widget types which can have widgets attached to them.
pub trait RelmContainerExt {
    /// Add widget as child to container.
    fn container_add(&self, widget: &impl AsRef<gtk::Widget>);
}

impl<T: RelmSetChildExt> RelmContainerExt for T {
    fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
        self.container_set_child(Some(widget));
    }
}

macro_rules! append_impl {
    ($($type:ty),+) => {
        $(
            impl RelmContainerExt for $type {
                fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
                    self.append(widget.as_ref());
                }
            }
        )+
    }
}

macro_rules! add_child_impl {
    ($($type:ty),+) => {
        $(
            impl RelmContainerExt for $type {
                fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
                    self.add_child(widget.as_ref());
                }
            }
        )+
    }
}

append_impl!(gtk::Box, gtk::ListBox);
add_child_impl!(gtk::InfoBar, gtk::Stack);
