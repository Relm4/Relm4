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
    /// Iterates across the child widgets of a widget
    fn iter_children(&self) -> Box<dyn Iterator<Item = gtk::Widget>>;

    /// Iterates children of a widget with a closure.
    fn for_each_child<F: FnMut(gtk::Widget) + 'static>(&self, func: F);
}

impl<T: gtk::glib::IsA<gtk::Widget>> RelmWidgetExt for T {
    fn for_each_child<F: FnMut(gtk::Widget) + 'static>(&self, mut func: F) {
        let mut widget = self.first_child();

        while let Some(child) = widget.take() {
            widget = child.next_sibling();
            func(child);
        }
    }

    fn iter_children(&self) -> Box<dyn Iterator<Item = gtk::Widget>> {
        let mut widget = self.first_child();

        let iterator = std::iter::from_fn(move || {
            if let Some(child) = widget.take() {
                widget = child.next_sibling();
                return Some(child);
            }

            None
        });

        Box::new(iterator)
    }
}
