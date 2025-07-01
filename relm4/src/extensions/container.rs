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

#[cfg(any(feature = "libadwaita", feature = "libpanel"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "libadwaita", feature = "libpanel"))))]
macro_rules! add_impl {
    ($($type:ty: $child:ty), +) => {
        $(
            impl RelmContainerExt for $type {
                fn container_add(&self, child: &impl AsRef<$child>) {
                    self.add(child.as_ref());
                }
            }
        )+
    };
    ($($type:ty), +) => {
        $(
            impl RelmContainerExt for $type {
                fn container_add(&self, widget: &impl AsRef<gtk::Widget>) {
                    self.add(widget.as_ref());
                }
            }
        )+
    };
}

append_impl!(gtk::Box, gtk::ListBox);
add_child_impl!(gtk::InfoBar, gtk::Stack);

#[cfg(feature = "gnome_42")]
#[cfg_attr(docsrs, doc(cfg(feature = "gnome_42")))]
append_impl!(gtk::FlowBox);

#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
mod libadwaita {
    use super::RelmContainerExt;
    use adw::prelude::{PreferencesGroupExt, PreferencesPageExt};
    append_impl!(adw::Leaflet, adw::Carousel, adw::TabView);
    add_impl! {
        adw::PreferencesPage: adw::PreferencesGroup
    }
    add_impl! {
        adw::PreferencesGroup,
        adw::Squeezer
    }

    #[cfg(all(feature = "libadwaita", feature = "gnome_45"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "libadwaita", feature = "gnome_45"))))]
    add_impl! {
        adw::NavigationView: adw::NavigationPage
    }
}

#[cfg(feature = "libpanel")]
#[cfg_attr(docsrs, doc(cfg(feature = "libpanel")))]
mod libpanel {
    use super::RelmContainerExt;
    use panel::prelude::PanelFrameExt;
    append_impl!(panel::Paned);
    add_impl! {
        panel::Frame: panel::Widget
    }
}
