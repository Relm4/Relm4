use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};

use crate::component::Component;
use crate::component::ComponentController;
use crate::Application;
use crate::ComponentBuilder;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp {
    /// The [`Application`] that's used internally to setup
    /// and run your application.
    pub app: Application,
}

impl RelmApp {
    /// Create a Relm4 application.
    #[must_use]
    pub fn new(app_id: &str) -> Self {
        crate::init();

        let app = crate::main_application();
        app.set_application_id(Some(app_id));

        Self { app }
    }

    /// Create a Relm4 application.
    pub fn with_app(app: impl IsA<Application> + Cast) -> Self {
        crate::init();

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

        let RelmApp { app } = self;

        let payload = Cell::new(Some(payload));

        app.connect_activate(move |app| {
            if let Some(payload) = payload.take() {
                assert!(
                    app.is_registered(),
                    "App should be already registered when activated"
                );

                let builder = ComponentBuilder::<C>::default();
                let controller = builder.launch(payload).detach();
                let window = controller.widget().clone();

                app.add_window(window.as_ref());
                window.show();
            } else {
                panic!("Can't start Relm4 applications twice");
            }
        });

        app.run_with_args(args);
    }
}
