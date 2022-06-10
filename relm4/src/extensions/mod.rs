mod container;
mod iter_children;
mod remove;
mod set_child;

#[cfg(test)]
mod tests;

#[allow(unreachable_pub)]
pub use self::container::RelmContainerExt;
#[allow(unreachable_pub)]
pub use self::iter_children::RelmIterChildrenExt;
#[allow(unreachable_pub)]
pub use self::remove::{RelmRemoveAllExt, RelmRemoveExt};
#[allow(unreachable_pub)]
pub use self::set_child::RelmSetChildExt;

use gtk::prelude::*;

/// Additional methods for `gtk::builders::ApplicationBuilder`
pub trait ApplicationBuilderExt {
    /// Convenience method for launching an application and initializing the window.
    fn launch<F>(self, init: F)
    where
        F: Fn(gtk::Application, gtk::ApplicationWindow) + 'static;
}

impl ApplicationBuilderExt for gtk::builders::ApplicationBuilder {
    fn launch<F>(self, init: F)
    where
        F: Fn(gtk::Application, gtk::ApplicationWindow) + 'static,
    {
        let app = self.build();

        app.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::new(app);

            init(app.clone(), window.clone());

            window.show();
        });

        app.run();
    }
}

/// Additional methods for `gtk::Widget`
pub trait RelmWidgetExt {
    /// Attach widget to a `gtk::SizeGroup`.
    fn set_size_group(&self, size_group: &gtk::SizeGroup);

    /// Locate the top level window this widget is attached to.
    ///
    /// Equivalent to `widget.ancestor(gtk::Window::static_type())`, then casting.
    fn toplevel_window(&self) -> Option<gtk::Window>;
}

impl<T: gtk::glib::IsA<gtk::Widget>> RelmWidgetExt for T {
    fn set_size_group(&self, size_group: &gtk::SizeGroup) {
        size_group.add_widget(self);
    }

    fn toplevel_window(&self) -> Option<gtk::Window> {
        self.ancestor(gtk::Window::static_type())
            .and_then(|widget| widget.dynamic_cast::<gtk::Window>().ok())
    }
}

/// Additional methods for `gtk::ListBox`.
pub trait RelmListBoxExt {
    /// Get the index of a widget attached to a listbox.
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32>;

    /// Remove the row of a child attached a listbox.
    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>);

    /// Get the row of a widget attached to a listbox.
    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow>;
}

impl RelmListBoxExt for gtk::ListBox {
    fn index_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<i32> {
        self.row_of_child(widget).map(|row| row.index())
    }

    fn remove_row_of_child(&self, widget: &impl AsRef<gtk::Widget>) {
        if let Some(row) = self.row_of_child(widget) {
            row.set_child(None::<&gtk::Widget>);
            self.remove(&row);
        }
    }

    fn row_of_child(&self, widget: &impl AsRef<gtk::Widget>) -> Option<gtk::ListBoxRow> {
        if let Some(row) = widget.as_ref().ancestor(gtk::ListBoxRow::static_type()) {
            if let Some(row) = row.downcast_ref::<gtk::ListBoxRow>() {
                if let Some(parent_widget) = row.parent() {
                    if let Some(parent_box) = parent_widget.downcast_ref::<gtk::ListBox>() {
                        if parent_box == self {
                            return Some(row.clone());
                        }
                    }
                }
            }
        }

        None
    }
}

/// Type of children inside a container.
///
/// For example, `gtk::ListBox` only contains `gtk::ListBoxRow` widgets
/// as children. If you add any other kind of widget, a row is automatically
/// inserted between the list box and the widget.
///
/// For simple widgets like `gtk::Box`, the children type will be `gtk::Widget`,
/// meaning that it can be any widget type.
pub trait ContainerChild {
    /// Type of container children.
    type Child: IsA<gtk::Widget>;
}

macro_rules! container_child_impl {
    ($($type:ty: $child:ty)+) => {
        $(
            impl ContainerChild for $type {
                type Child = $child;
            }
        )+
    };
    ($($type:ty)+) => {
        $(
            impl ContainerChild for $type {
                type Child = gtk::Widget;
            }
        )+
    };
}

container_child_impl! {
    gtk::Box
    gtk::Fixed
    gtk::Grid
    gtk::ActionBar
    gtk::Stack
    gtk::HeaderBar
    gtk::InfoBar
    gtk::Button
    gtk::ComboBox
    gtk::FlowBoxChild
    gtk::Frame
    gtk::Popover
    gtk::Window
    gtk::ApplicationWindow
    gtk::ListBoxRow
    gtk::ScrolledWindow
    gtk::Dialog
    gtk::LinkButton
    gtk::ToggleButton
    gtk::Overlay
    gtk::Revealer
}

container_child_impl! {
    gtk::ListBox: gtk::ListBoxRow
    gtk::FlowBox: gtk::FlowBoxChild
}

#[cfg(feature = "libadwaita")]
mod libadwaita {
    use super::ContainerChild;

    container_child_impl! {
        adw::TabView
        adw::Window
        adw::Bin
        adw::ApplicationWindow
        adw::Clamp
        adw::ClampScrollable
        adw::SplitButton
        adw::StatusPage
    }
}
