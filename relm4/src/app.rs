use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};

use crate::component::Component;
use crate::component::ComponentController;
use crate::Application;
use crate::ComponentBuilder;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp {
    /// The application that's used internally to setup
    /// and run the application.
    ///
    /// Depending on your feature flag this is either
    /// [`gtk::Application`] or [`adw::Application`].
    pub app: Application,
}

impl RelmApp
{
    /// Create a Relm4 application.
    pub fn new(app_id: &str) -> Self {
        let app = Application::builder().application_id(app_id).build();

        Self::with_app(app)
    }

    /// Create a Relm4 application.
    pub fn with_app(app: impl IsA<Application> + Cast) -> Self {
        gtk::init().unwrap();

        #[cfg(feature = "libadwaita")]
        adw::init();

        let app = app.upcast();

        Self { app }
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::prelude::ApplicationExtManual::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run<C>(self, payload: C::InitParams)
        where
            C: Component,
            C::Root: IsA<gtk::Window> + WidgetExt,
     {
        self.run_with_args::<C, &str>(payload, &[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<C, S>(self, payload: C::InitParams, args: &[S])
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
                if !app.is_registered() {
                    panic!("App should be already registered when activated");
                }

                let bridge = ComponentBuilder::<C>::new();
                let controller = bridge.launch(payload).detach();
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
