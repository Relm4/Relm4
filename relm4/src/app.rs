use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};

use crate::component::Component;
use crate::component::ComponentController;
use crate::ComponentBuilder;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp {
    /// The [`gtk::Application`] that's used internally to setup
    /// and run your application.
    app: gtk::Application,
}

impl RelmApp {
    /// Create a new Relm4 application.
    ///
    /// This function will create a new [`gtk::Application`] object if necessary.
    ///
    /// If the `libadwaita` feature is enabled, then the created [`gtk::Application`] will be an
    /// instance of [`adw::Application`]. This can be overridden by passing your own application
    /// object to [`RelmApp::with_app`].
    #[must_use]
    pub fn new(app_id: &str) -> Self {
        let app = crate::main_application();
        app.set_application_id(Some(app_id));

        Self { app }
    }

    /// Create a Relm4 application with a provided [`gtk::Application`].
    pub fn with_app(app: impl IsA<gtk::Application>) -> Self {
        let app = app.upcast();
        crate::set_main_application(app.clone());

        Self { app }
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::prelude::ApplicationExtManual::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run<C>(self, payload: C::Init)
    where
        C: Component,
        C::Root: IsA<gtk::Window> + WidgetExt,
    {
        self.run_with_args::<C, &str>(payload, &[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<C, S>(self, payload: C::Init, args: &[S])
    where
        C: Component,
        C::Root: IsA<gtk::Window> + WidgetExt,
        S: AsRef<str>,
    {
        use std::cell::Cell;

        let Self { app } = self;

        let payload = Cell::new(Some(payload));

        app.connect_activate(move |app| {
            if let Some(payload) = payload.take() {
                assert!(
                    app.is_registered(),
                    "App should be already registered when activated"
                );

                let builder = ComponentBuilder::<C>::default();
                let connector = builder.launch(payload);

                // Run late initialization for transient windows for example.
                crate::late_initialization::run_late_init();

                let window = connector.detach().widget().clone();

                app.add_window(window.as_ref());
                window.show();
            } else {
                panic!("Can't start Relm4 applications twice");
            }
        });

        app.run_with_args(args);
    }
}
