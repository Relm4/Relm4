use glib::Sender;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt};

use std::marker::PhantomData;

pub mod generator;

/// Relm app that runs the main application.
/// The app consists of widgets that represents the UI and the model
/// that stores the application state.
/// A relm app might run as a standalone application or may consist
/// of multiple components that communicate with each other.
/// Use [`RelmApp::create()`] to create the app and call `run()` on it
/// to start the application.
#[derive(Clone)]
pub struct RelmApp<Widgets, Model, Msg>
where
    Widgets: Widget<Msg, Model>,
    Model: AppUpdate<Msg>,
{
    widgets: PhantomData<Widgets>,
    model: PhantomData<Model>,
    sender: Sender<Msg>,
    app: gtk::Application,
}

impl<Widgets, Model, Msg> RelmApp<Widgets, Model, Msg>
where
    Widgets: Widget<Msg, Model, Root = gtk::ApplicationWindow> + 'static,
    Model: AppUpdate<Msg, Widgets = Widgets> + 'static,
{
    /// Run the application. This will return once the application is closed.
    pub fn run(&self) {
        self.app.run();
    }

    /// Create an application.
    pub fn create() -> Self {
        let app = gtk::ApplicationBuilder::new().build();
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model();

        let mut widgets: Widgets = Widgets::init_view(sender.clone(), &model);
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
                model.update(msg, &widgets);
                model.view(&mut widgets);
                glib::Continue(true)
            });
        }

        RelmApp {
            widgets: PhantomData,
            model: PhantomData,
            sender,
            app,
        }
    }
}

/// Component that can be part of the main application or other
/// components. Components can send each other messages and have their own
/// widgets, models and message type. They also store the parent message type
/// to communicate with the parent.
#[derive(Clone)]
pub struct RelmComponent<Widgets, Model, Msg, ParentMsg>
where
    Widgets: Widget<Msg, Model>,
    Model: ComponentUpdate<Msg, ParentMsg>,
{
    widgets: PhantomData<Widgets>,
    model: PhantomData<Model>,
    parent_msg: PhantomData<ParentMsg>,
    sender: Sender<Msg>,
}

impl<Widgets, Model, Msg, ParentMsg> RelmComponent<Widgets, Model, Msg, ParentMsg>
where
    Widgets: Widget<Msg, Model> + 'static,
    Model: ComponentUpdate<Msg, ParentMsg, Widgets = Widgets> + 'static,
    ParentMsg: 'static,
{
    /// Create component. Usually you can store Self in the widgets of the parent component.
    /// The root widget needs to be attached to a GTK container in the parent's `init_view` function.
    pub fn create(parent_sender: Sender<ParentMsg>) -> (Self, Widgets::Root) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model();

        let mut widgets: Widgets = Widgets::init_view(sender.clone(), &model);
        let root = widgets.root_widget();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Msg| {
                model.update(msg, &widgets, parent_sender.clone());
                model.view(&mut widgets);
                glib::Continue(true)
            });
        }

        (
            RelmComponent {
                widgets: PhantomData,
                model: PhantomData,
                parent_msg: PhantomData,
                sender,
            },
            root,
        )
    }

    /// Get a sender that can send messages to this component.
    pub fn sender(&self) -> Sender<Msg> {
        self.sender.clone()
    }
}

/// Widgets are part of an app or components. They represent the UI
/// that usually consists out of GTK widgets. The root represents the
/// widget that all other widgets are attached to.
/// The root of the main app must be a [`gtk::ApplicationWindow`].
pub trait Widget<Msg, Model> {
    type Root: WidgetExt;

    /// Initialize the UI.
    fn init_view(sender: glib::Sender<Msg>, model: &Model) -> Self;

    /// Return the root widget.
    fn root_widget(&self) -> Self::Root;
}

/// Methods that initialize and update the main app.
pub trait AppUpdate<Msg> {
    type Widgets;

    /// Create initial model.
    fn init_model() -> Self;

    /// Update the model.
    fn update(&mut self, msg: Msg, widgets: &Self::Widgets);

    /// Update the view to represent the updated model.
    fn view(&self, widgets: &mut Self::Widgets);
}

/// Methods that initialize and update a component.
pub trait ComponentUpdate<Msg, ParentMsg> {
    type Widgets;

    /// Create initial model.
    fn init_model() -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    fn update(&mut self, msg: Msg, widgets: &Self::Widgets, parent_sender: Sender<ParentMsg>);

    /// Update the view to represent the updated model.
    fn view(&self, widgets: &mut Self::Widgets);
}
