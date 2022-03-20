use gtk::prelude::*;

/// Widget types which allow to set or unset their child.
pub trait RelmSetChildExt {
    /// Set a child for the container or remove it using [`None`].
    fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>);
}

macro_rules! set_child_impl {
    ($($type:ty),+) => {
        $(
            impl RelmSetChildExt for $type {
                fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>) {
                    self.set_child(widget.map(|w| w.as_ref()));
                }
            }
        )+
    }
}

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

#[cfg(feature = "libadwaita")]
mod libadwaita {
    use super::RelmSetChildExt;
    use adw::prelude::AdwWindowExt;

    impl RelmSetChildExt for adw::Window {
        fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>) {
            self.set_content(widget.map(|w| w.as_ref()));
        }
    }
}
