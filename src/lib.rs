use glib::Sender;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use std::marker::PhantomData;

pub mod generator;
mod traits;

pub use traits::*;

/// Relm app that runs the main application.
/// The app consists of widgets that represents the UI and the model
/// that stores the application state.
/// A relm app might run as a standalone application or may consist
/// of multiple components that communicate with each other.
/// Use [`RelmApp::create()`] to create the app and call `run()` on it
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
    Model: AppUpdate<Components, Msg, Widgets = Widgets> + 'static,
    Components: RelmComponents<Model, Msg> + 'static,
    Msg: 'static,
{
    /// Run the application. This will return once the application is closed.
    pub fn run(&self) {
        self.app.run();
    }

    /// Create an application.
    pub fn new(mut model: Model) -> Self {
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
                model.view(&mut widgets, sender.clone());
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

/// Component that can be part of the main application or other
/// components. Components can send each other messages and have their own
/// widgets, models and message type. They also store the parent message type
/// to communicate with the parent.
#[derive(Clone)]
pub struct RelmComponent<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
where
    Widgets: RelmWidgets<Model, Components, Msg>,
    Model: ComponentUpdate<Components, Msg, ParentModel, ParentMsg>,
{
    widgets: PhantomData<Widgets>,
    model: PhantomData<Model>,
    parent_info: PhantomData<(ParentModel, ParentMsg)>,
    components: PhantomData<Components>,
    sender: Sender<Msg>,
    root_widget: Widgets::Root,
}

impl<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
    RelmComponent<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
where
    Widgets: RelmWidgets<Model, Components, Msg> + 'static,
    Model: ComponentUpdate<Components, Msg, ParentModel, ParentMsg, Widgets = Widgets> + 'static,
    Components: RelmComponents<Model, Msg> + 'static,
    ParentMsg: 'static,
    Msg: 'static,
{
    /// Create component. Usually you can store Self in the widgets of the parent component.
    /// The root widget needs to be attached to a GTK container in the parent's `init_view` function.
    pub fn new(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Components::init_components(&model, sender.clone());
        let cloned_sender = sender.clone();

        let mut widgets: Widgets = Widgets::init_view(&model, &components, sender.clone());
        let root_widget = widgets.root_widget();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                model.view(&mut widgets, sender.clone());
                glib::Continue(true)
            });
        }

        RelmComponent {
            widgets: PhantomData,
            model: PhantomData,
            parent_info: PhantomData,
            components: PhantomData,
            sender: cloned_sender,
            root_widget,
        }
    }

    /// Get a sender that can send messages to this component.
    pub fn sender(&self) -> Sender<Msg> {
        self.sender.clone()
    }

    pub fn root_widget(&self) -> &Widgets::Root {
        &self.root_widget
    }
}

pub fn spawn_future<F: futures_core::future::Future<Output = ()> + Send + 'static>(f: F) {
    glib::MainContext::ref_thread_default().spawn(f);
}
