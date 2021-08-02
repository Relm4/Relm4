use gtk::glib::{self, Sender};

use std::marker::{PhantomData, Send};
use std::sync::mpsc::channel;

use crate::{ComponentUpdate, RelmComponents, RelmWidgets};

/// Component that can be part of the main application or other
/// components. Components can send each other messages and have their own
/// widgets, models and message type. They also store the parent message type
/// to communicate with the parent.
#[derive(Clone)]
pub struct RelmComponent<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
where
    Widgets: RelmWidgets<Model = Model, Components = Components, Msg = Msg>,
    Model: ComponentUpdate<
        Components = Components,
        Msg = Msg,
        ParentModel = ParentModel,
        ParentMsg = ParentMsg,
    >,
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
    Widgets: RelmWidgets<Model = Model, Components = Components, Msg = Msg> + 'static,
    Model: ComponentUpdate<
            Components = Components,
            Msg = Msg,
            ParentModel = ParentModel,
            ParentMsg = ParentMsg,
        > + 'static,
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
                widgets.view(&model, sender.clone());
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

    pub fn send(&self, msg: Msg) -> Result<(), std::sync::mpsc::SendError<Msg>> {
        self.sender.send(msg)
    }

    pub fn root_widget(&self) -> &Widgets::Root {
        &self.root_widget
    }
}

impl<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
    RelmComponent<Widgets, Model, Components, Msg, ParentModel, ParentMsg>
where
    Widgets: RelmWidgets<Model = Model, Components = Components, Msg = Msg> + 'static,
    Model: ComponentUpdate<
            Components = Components,
            Msg = Msg,
            ParentModel = ParentModel,
            ParentMsg = ParentMsg,
        > + Send
        + 'static,
    Components: RelmComponents<Model, Msg> + Send + 'static,
    ParentMsg: Send + 'static,
    Msg: Send + 'static,
{
    pub fn with_new_thread(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self {
        let (global_sender, global_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let model = Model::init_model(parent_model);

        let components = Components::init_components(&model, sender.clone());
        let cloned_sender = sender.clone();

        let mut widgets: Widgets = Widgets::init_view(&model, &components, sender.clone());
        let root_widget = widgets.root_widget();

        let update_sender = sender.clone();
        let view_sender = sender;

        let (model_tx, model_rx) = channel();
        model_tx.send(model).unwrap();

        std::thread::spawn(move || {
            let context = glib::MainContext::new();
            context.push_thread_default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Msg| {
                let mut model: Model = model_rx.recv().unwrap();
                model.update(
                    msg,
                    &components,
                    update_sender.clone(),
                    parent_sender.clone(),
                );
                global_sender.send(model).unwrap();
                glib::Continue(true)
            });

            let main_loop = glib::MainLoop::new(Some(&context), true);
            main_loop.run();
            context.pop_thread_default();
        });

        let global_context = glib::MainContext::default();
        let _global_guard = global_context.acquire().unwrap();
        global_receiver.attach(Some(&global_context), move |model: Model| {
            widgets.view(&model, view_sender.clone());
            model_tx.send(model).unwrap();
            glib::Continue(true)
        });

        RelmComponent {
            widgets: PhantomData,
            model: PhantomData,
            parent_info: PhantomData,
            components: PhantomData,
            sender: cloned_sender,
            root_widget,
        }
    }
}
