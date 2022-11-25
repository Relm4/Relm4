use super::ContainerChild;
use gtk::prelude::*;

/// Widget types which allow to set or unset their child.
pub trait RelmSetChildExt: ContainerChild {
    /// Set a child for the container or remove it using [`None`].
    fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>);

    /// Get the child of a container.
    /// Returns [`None`] if the container has no child.
    fn container_get_child(&self) -> Option<gtk::Widget>;
}

macro_rules! set_child_impl {
    ($($type:ty),+) => {
        $(
            impl RelmSetChildExt for $type {
                fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>) {
                    self.set_child(widget.map(|w| w.as_ref()));
                }

                fn container_get_child(&self) -> Option<gtk::Widget> {
                    self.child()
                }
            }
        )+
    }
}

set_child_impl!(
    gtk::Button,
    gtk::LinkButton,
    gtk::ToggleButton,
    gtk::FlowBoxChild,
    gtk::Frame,
    gtk::ListBoxRow,
    gtk::Popover,
    gtk::Window,
    gtk::ScrolledWindow,
    gtk::ApplicationWindow,
    gtk::Overlay,
    gtk::Revealer,
    gtk::WindowHandle
);

#[cfg(feature = "libadwaita")]
mod libadwaita {
    use super::RelmSetChildExt;
    use adw::prelude::{AdwApplicationWindowExt, AdwWindowExt, BinExt};

    macro_rules! set_child_content_impl {
        ($($type:ty),+) => {
            $(
                impl RelmSetChildExt for $type {
                    fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>) {
                        self.set_content(widget.map(|w| w.as_ref()));
                    }

                    fn container_get_child(&self) -> Option<gtk::Widget> {
                        self.content()
                    }
                }
            )+
        }
    }

    set_child_content_impl!(adw::Window, adw::ApplicationWindow);
    set_child_impl!(
        adw::Bin,
        adw::Clamp,
        adw::ClampScrollable,
        adw::SplitButton,
        adw::StatusPage,
        adw::ToastOverlay
    );
}
