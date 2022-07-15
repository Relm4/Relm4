use gtk::prelude::{ApplicationExt, ApplicationExtManual, Cast, GtkApplicationExt, IsA, WidgetExt};

use crate::component::Component;
use crate::component::ComponentController;
use crate::Application;
use crate::ComponentBuilder;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp<C: Component> {
    bridge: ComponentBuilder<C>,

    /// The application that's used internally to setup
    /// and run the application.
    ///
    /// Depending on your feature flag this is either
    /// [`gtk::Application`] or [`adw::Application`].
    pub app: Application,
}

impl<C: Component> RelmApp<C>
where
    C::Root: IsA<gtk::Window> + WidgetExt,
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
        app.register(gtk::gio::Cancellable::NONE).expect("");

        let bridge = ComponentBuilder::<C>::new();

        Self { bridge, app }
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::prelude::ApplicationExtManual::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run(self, payload: C::InitParams) {
        self.run_with_args::<&str>(payload, &[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<S>(self, payload: C::InitParams, args: &[S])
    where
        S: AsRef<str>,
    {
        use std::cell::Cell;

        let RelmApp { bridge, app } = self;
        let payload = Cell::new(Some(payload));
        let bridge = Cell::new(Some(bridge));

        app.connect_activate(move |app| {
            if let (Some(bridge), Some(payload)) = (bridge.take(), payload.take()) {
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
