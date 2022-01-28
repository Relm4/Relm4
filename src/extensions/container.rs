use gtk::prelude::*;

/// Widget types which can have widgets attached to them.
pub trait RelmContainerExt {
    /// Add widget as child to container.
    fn container_add(&self, widget: &impl AsRef<gtk::Widget>);
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

macro_rules! set_child_impl {
    ($($type:ty),+) => {
        $(
            impl RelmContainerExt for $type {
                fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
                    self.set_child(Some(widget.as_ref()));
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
set_child_impl!(
    gtk::Button,
    gtk::ComboBox,
    gtk::FlowBoxChild,
    gtk::Frame,
    gtk::Popover,
    gtk::Window,
    gtk::ApplicationWindow,
    gtk::Dialog
);
add_child_impl!(gtk::InfoBar, gtk::Stack);
