use crate::{RelmSetChildExt, RelmWidgetExt};
use gtk::prelude::*;

/// Widget types which can have widgets removed from them.
pub trait RelmRemovableExt {
    /// Type of children of the container.
    type Child;
    /// Removes the widget from the container
    /// if it is a child of the container.
    fn container_remove(&self, widget: &impl AsRef<Self::Child>);
    /// Remove all children from the container.
    fn remove_all(&self);
}

impl<'a, T: RelmSetChildExt> RelmRemovableExt for T {
    type Child = gtk::Widget;
    fn container_remove(&self, _widget: &impl AsRef<Self::Child>) {
        self.container_set_child(None::<&gtk::Widget>);
    }
    fn remove_all(&self) {
        self.container_set_child(None::<&gtk::Widget>);
    }
}

impl RelmRemovableExt for gtk::ListBox {
    type Child = gtk::ListBoxRow;
    fn container_remove(&self, widget: &impl AsRef<Self::Child>) {
        self.remove(widget.as_ref());
    }
    fn remove_all(&self) {
        while let Some(child) = self.last_child() {
            self.remove(&child);
        }
    }
}

impl RelmRemovableExt for gtk::HeaderBar {
    type Child = gtk::Widget;
    fn container_remove(&self, widget: &impl AsRef<Self::Child>) {
        self.remove(widget.as_ref());
    }
    /*
    To remove all children, assume the following widget structure:

    HeaderBar
    ╰── WindowHandle
        ╰── CenterBox
            ├── Box
            │   ├── WindowControls
            │   ╰── [other children] <- to remove
            |
            ├── [Title Widget]
            ╰── Box
                ├── [other children] <- to remove
                ╰── WindowControls
    */
    fn remove_all(&self) {
        let handle = self
            .first_child()
            .expect("The `HeaderBar` has no children.")
            .downcast::<gtk::WindowHandle>()
            .expect("The child of `HeaderBar` is not a `WindowHandle`.");

        let center_box = handle
            .first_child()
            .expect("The `WindowHandle` has no children.")
            .downcast::<gtk::CenterBox>()
            .expect("The child of `WindowHandle` is not a `CenterBox`.");

        let start_box = center_box
            .first_child()
            .expect("The `CenterBox` has no children.")
            .downcast::<gtk::Box>()
            .expect("The first child of `CenterBox` is not a `Box`.");

        let title_widget = start_box
            .next_sibling()
            .expect("The `CenterBox` has only one child.");

        let end_box = title_widget
            .next_sibling()
            .expect("The `CenterBox` has only two children.")
            .downcast::<gtk::Box>()
            .expect("The third child of `CenterBox` is not a `Box`.");

        let mut start_children = start_box.iter_children();
        let mut end_children = end_box.iter_children_reverse();

        let _start_controls = start_children
            .next()
            .expect("The start `Box` has no children.")
            .downcast::<gtk::WindowControls>()
            .expect("The first child of the start `Box` is not `WindowControls`.");

        let _end_controls = end_children
            .next()
            .expect("The end `Box` has no children.")
            .downcast::<gtk::WindowControls>()
            .expect("The last child of the end `Box` is not `WindowControls`.");

        for child in start_children {
            start_box.remove(&child);
        }

        for child in end_children {
            end_box.remove(&child);
        }
    }
}

impl RelmRemovableExt for gtk::ActionBar {
    type Child = gtk::Widget;
    fn container_remove(&self, widget: &impl AsRef<Self::Child>) {
        self.remove(widget.as_ref());
    }
    /*
    To remove all children, assume the following widget structure:

    ActionBar
    ╰── Revealer
        ╰── CenterBox
            ├── Box
            │   ╰── [start children] <- to remove
            ├── (Optional) [center widget]
            ╰── Box
                ╰── [end children] <- to remove
    */
    fn remove_all(&self) {
        let revealer = self
            .first_child()
            .expect("The `ActionBar` has no children.")
            .downcast::<gtk::Revealer>()
            .expect("The child of `ActionBar` is not a `Revealer`.");

        let center_box = revealer
            .first_child()
            .expect("The `Revealer` has no children.")
            .downcast::<gtk::CenterBox>()
            .expect("The child of `Revealer` is not a `CenterBox`.");

        let start_box = center_box
            .first_child()
            .expect("The `CenterBox` has no children.")
            .downcast::<gtk::Box>()
            .expect("The first child of `CenterBox` is not a `Box`.");

        let second_widget = start_box
            .next_sibling()
            .expect("The `CenterBox` has only one child.");

        let third_widget = second_widget.next_sibling();

        // Third widget exists: therefore, the `second_widget` is the center widget,
        // and the `third_widget` is the end box.
        let end_box = if let Some(widget) = third_widget {
            widget
                .downcast::<gtk::Box>()
                .expect("The third child of `CenterBox` is not a `Box`.")
        }
        // Third widget does not exist: therefore, the center widget is not setup,
        // and the `second_widget` is the end box.
        else {
            second_widget
                .downcast::<gtk::Box>()
                .expect("The second child of `CenterBox` is not a `Box`.")
        };

        for child in start_box.iter_children() {
            start_box.remove(&child);
        }

        for child in end_box.iter_children_reverse() {
            end_box.remove(&child);
        }
    }
}

macro_rules! remove_impl {
    ($($type:ty),+) => {
        $(
            impl RelmRemovableExt for $type {
                type Child = gtk::Widget;
                fn container_remove(&self, widget: &impl AsRef<Self::Child>) {
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
            impl RelmRemovableExt for $type {
                type Child = gtk::Widget;
                fn container_remove(&self, widget: &impl AsRef<Self::Child>) {
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

remove_impl!(gtk::Box, gtk::Fixed, gtk::Grid, gtk::FlowBox, gtk::Stack);
remove_child_impl!(gtk::InfoBar);
