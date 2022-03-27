use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkApplicationExt, IsA, WidgetExt};

use crate::component::Component;
use crate::component::ComponentController;
use crate::ComponentBuilder;

/// An app that runs the main application.
#[derive(Debug)]
pub struct RelmApp<C: Component> {
    bridge: ComponentBuilder<C>,

    /// The [`gtk::Application`] that's used internally to setup
    /// and run the application.
    pub app: gtk::Application,
}

impl<C: Component> RelmApp<C>
where
    C::Root: IsA<gtk::Window> + WidgetExt,
{
    /// Create a Relm4 application.
    pub fn new(app_id: &str) -> Self {
        gtk::init().expect("Couldn't initialize GTK");

        let app = gtk::Application::builder().application_id(app_id).build();

        let bridge = ComponentBuilder::<C>::new();

        Self { bridge, app }
    }

    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::Application::run`], this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run(self, payload: C::InitParams) {
        let RelmApp { bridge, app } = self;
        let controller = bridge.launch(payload).detach();
        let window = controller.widget().clone();

        app.connect_activate(move |app| {
            app.add_window(&window);
            window.show();
        });

        app.run_with_args::<&str>(&[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<S>(self, payload: C::InitParams, args: &[S])
    where
        S: AsRef<str>,
    {
        let RelmApp { bridge, app } = self;
        let controller = bridge.launch(payload).detach();
        let window = controller.widget().clone();

        app.connect_activate(move |app| {
            app.add_window(&window);
            window.show();
        });

        app.run_with_args(args);
    }
}
