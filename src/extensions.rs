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
