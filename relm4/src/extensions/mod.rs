mod container;
mod iter_children;
mod remove;
mod set_child;

#[cfg(test)]
mod tests;
mod widget_ext;

pub use container::RelmContainerExt;
pub use iter_children::RelmIterChildrenExt;
pub use remove::{RelmRemoveAllExt, RelmRemoveExt};
pub use set_child::RelmSetChildExt;
pub use widget_ext::RelmWidgetExt;

use gtk::prelude::{
    ApplicationExt, ApplicationExtManual, Cast, IsA, ListBoxRowExt, StaticType, WidgetExt,
};

/// Get a reference to a widget.
///
/// This trait is an extension of [`AsRef`]
/// that always returns `&`[`gtk::Widget`].
pub trait WidgetRef {
    /// Returns a reference to a widget.
    ///
    /// Like [`AsRef::as_ref`] it will auto-dereference.
    fn widget_ref(&self) -> &gtk::Widget;
}

impl<T: AsRef<gtk::Widget>> WidgetRef for T {
    fn widget_ref(&self) -> &gtk::Widget {
        self.as_ref()
    }
}

pub trait WidgetTemplate: Sized + std::fmt::Debug + std::ops::Deref<Target = Self::Widget> {
    type Widget;

    fn init() -> Self;
}

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
                    if let Some(parent_box) = parent_widget.downcast_ref::<Self>() {
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
        adw::PreferencesGroup
        adw::ToastOverlay
    }
}
