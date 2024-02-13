#![allow(deprecated)]
use gtk::prelude::*;

use crate::{ContainerChild, RelmSetChildExt};

/// Widget types which can have widgets attached to them.
pub trait RelmContainerExt: ContainerChild {
    /// Add widget as child to container.
    fn container_add(&self, widget: &impl AsRef<Self::Child>);
}

impl<T: RelmSetChildExt> RelmContainerExt for T {
    fn container_add(&self, widget: &impl AsRef<T::Child>) {
        self.container_set_child(Some(widget));
    }
}

#[allow(deprecated)]
impl RelmContainerExt for gtk::Dialog {
    fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
        self.content_area().append(widget.as_ref());
    }
}

macro_rules! append_impl {
    ($($type:ty),+) => {
        $(
            impl RelmContainerExt for $type {
                #[allow(unused_qualifications)]
                fn container_add(&self, widget: &impl AsRef<<$type as crate::extensions::ContainerChild>::Child>) {
                    self.append(widget.as_ref());
                }
            }
        )+
    }
}

macro_rules! add_child_impl {
    ($($type:ty),+) => {
        $(
            #[allow(deprecated)]
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

#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
mod libadwaita {
    use super::RelmContainerExt;
    append_impl!(adw::Leaflet, adw::Carousel, adw::TabView);

    impl RelmContainerExt for adw::PreferencesGroup {
        fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
            use adw::prelude::PreferencesGroupExt;
            self.add(widget.as_ref());
        }
    }

    impl RelmContainerExt for adw::Squeezer {
        fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
            self.add(widget.as_ref());
        }
    }

    #[cfg(all(feature = "libadwaita", feature = "gnome_45"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "libadwaita", feature = "gnome_45"))))]
    impl RelmContainerExt for adw::NavigationView {
        fn container_add(&self, widget: &impl AsRef<adw::NavigationPage>) {
            self.add(widget.as_ref());
        }
    }
}
