use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use std::marker::PhantomData;

use crate::{AppUpdate, RelmComponents, RelmWidgets};

/// Relm app that runs the main application.
/// The app consists of widgets that represents the UI and the model
/// that stores the application state.
/// A relm app might run as a standalone application or may consist
/// of multiple components that communicate with each other.
/// Use [`RelmApp::new()`] to create the app and call `run()` on it
/// to start the application.
#[derive(Clone)]
pub struct RelmApp<Widgets, Model, Components, Msg>
where
    Widgets: RelmWidgets<Model, Components, Msg>,
    Model: AppUpdate<Components, Msg>,
{
    widgets: PhantomData<Widgets>,
    model: PhantomData<Model>,
    components: PhantomData<Components>,
    msg: PhantomData<Msg>,
    app: gtk::Application,
}

impl<Widgets, Model, Components, Msg> RelmApp<Widgets, Model, Components, Msg>
where
    Widgets: RelmWidgets<Model, Components, Msg, Root = gtk::ApplicationWindow> + 'static,
    Model: AppUpdate<Components, Msg> + 'static,
    Components: RelmComponents<Model, Msg> + 'static,
    Msg: 'static,
{
    /// Run the application. This will return once the application is closed.
    pub fn run(&self) {
        self.app.run();
    }

    /// Create an application.
    pub fn new(mut model: Model) -> Self {
        gtk::init().expect("Couln't initialize GTK");
        let app = gtk::ApplicationBuilder::new().build();
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let components = Components::init_components(&model, sender.clone());

        let mut widgets: Widgets = Widgets::init_view(&model, &components, sender.clone());
        let root = widgets.root_widget();

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

            // Register receiver on the main loop to wait for messages to update model and view the changes.
            receiver.attach(Some(&context), move |msg: Msg| {
                model.update(msg, &components, sender.clone());
                widgets.view(&model, sender.clone());
                glib::Continue(true)
            });
        }

        RelmApp {
            widgets: PhantomData,
            model: PhantomData,
            components: PhantomData,
            msg: PhantomData,
            app,
        }
    }
}
