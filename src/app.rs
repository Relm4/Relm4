use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkApplicationExt, IsA};

use crate::component::Component;
use crate::Bridge;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp<C: Component> {
    bridge: Bridge<C, C::Root>,

    /// The [`gtk::Application`] that's used internally to setup
    /// and run the application.
    pub app: gtk::Application,
}

impl<C: Component> RelmApp<C>
where
    C::Root: IsA<gtk::Window>,
{
    /// Create a Relm4 application.
    pub fn new(app_id: &str) -> Self {
        gtk::init().expect("Couldn't initialize GTK");

        let app = gtk::Application::builder().application_id(app_id).build();

        let bridge = C::init();
        app.add_window(&bridge.root);

        Self { bridge, app }
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::Application::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run(self, payload: C::Payload) {
        let RelmApp { bridge, app } = self;
        let _controller = bridge.launch(payload).detach();

        app.connect_activate(|_| {});

        app.run_with_args::<&str>(&[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<S>(self, payload: C::Payload, args: &[S])
    where
        S: AsRef<str>,
    {
        let RelmApp { bridge, app } = self;
        let _controller = bridge.launch(payload).detach();

        app.connect_activate(|_| {});

        app.run_with_args(args);
    }
}
