use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, IsA};

use std::marker::PhantomData;

use crate::{AppUpdate, Application, Components, Model as ModelTrait, Widgets as WidgetsTrait};

fn init() {
    gtk::init().expect("Couldn't initialize GTK");

    #[cfg(feature = "libadwaita")]
    adw::init();
}

/// The app that runs the main application.
/// A [`RelmApp`] consists of a model that stores the application state
/// and widgets that represent the UI.
///
/// A [`RelmApp`] might run as a standalone application or may contain
/// multiple components that communicate with each other.
#[derive(Debug)]
pub struct RelmApp<Model>
where
    Model: ModelTrait + AppUpdate + 'static,
    Model::Widgets: WidgetsTrait<Model, ()> + 'static,
    <Model::Widgets as WidgetsTrait<Model, ()>>::Root:
        IsA<gtk::ApplicationWindow> + IsA<gtk::Window>,
    Model::Components: Components<Model> + 'static,
{
    model: PhantomData<Model>,
    app: Application,
}

impl<Model> RelmApp<Model>
where
    Model: ModelTrait + AppUpdate + 'static,
    Model::Widgets: WidgetsTrait<Model, ()> + 'static,
    <Model::Widgets as WidgetsTrait<Model, ()>>::Root:
        IsA<gtk::ApplicationWindow> + IsA<gtk::Window>,
    Model::Components: Components<Model> + 'static,
{
    /// Runs the application, returns once the application is closed.
    ///
    /// Unlike [`gtk::Application::run`](../gtk4/struct.Application.html#method.run), this function
    /// does not handle command-line arguments. To pass arguments to GTK, use
    /// [`RelmApp::run_with_args`].
    pub fn run(&self) {
        self.app.run_with_args::<&str>(&[]);
    }

    /// Runs the application with the provided command-line arguments, returns once the application
    /// is closed.
    pub fn run_with_args<S>(&self, args: &[S])
    where
        S: AsRef<str>,
    {
        self.app.run_with_args(args);
    }

    /// Create a Relm4 application.
    pub fn new(model: Model) -> Self {
        let app = Application::default();

        Self::with_app(model, app)
    }

    /// Create a new Relm4 application with an existing [`gtk::Application`].
    pub fn with_app(mut model: Model, app: Application) -> Self {
        // Initialize GTK/Libadwaita
        init();

        crate::APP
            .set(fragile::Fragile::new(app.clone()))
            .expect("APP was already set");

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut components = Model::Components::init_components(&model, sender.clone());

        let mut widgets = Model::Widgets::init_view(&model, &components, sender.clone());
        let root = widgets.root_widget();

        components.connect_parent(&widgets);

        // Initialize GTK
        app.connect_activate(move |app| {
            root.set_application(Some(app));
            root.present();
        });

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            let app = app.clone();

            // Register receiver on the main loop to wait for messages to update model and view the changes.
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                if !model.update(msg, &components, sender.clone()) {
                    app.quit();
                    return glib::Continue(false);
                }
                widgets.view(&model, sender.clone());
                glib::Continue(true)
            });
        }

        RelmApp {
            model: PhantomData,
            app,
        }
    }
}
