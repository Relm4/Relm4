use gtk::glib::{self, Sender};

use std::marker::{PhantomData, Send};
use std::sync::mpsc::channel;

use crate::{ComponentUpdate, Components, Model as ModelTrait, Widgets as WidgetsTrait};

/// Component that can be part of the main application or other
/// components. Components can send each other messages and have their own
/// widgets, models and message type. They also store the parent message type
/// to communicate with the parent.
pub struct RelmComponent<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel> + 'static,
    ParentModel: ModelTrait,
    Model::Widgets: WidgetsTrait<Model, ParentModel> + 'static,
{
    widgets: PhantomData<Model>,
    parent_model: PhantomData<ParentModel>,
    sender: Sender<Model::Msg>,
    root_widget: <Model::Widgets as WidgetsTrait<Model, ParentModel>>::Root,
}

impl<Model, ParentModel> RelmComponent<Model, ParentModel>
where
    Model::Widgets: WidgetsTrait<Model, ParentModel> + 'static,
    ParentModel: ModelTrait,
    Model: ComponentUpdate<ParentModel> + 'static,
{
    /// Create component. Usually you can store Self in the widgets of the parent component.
    /// The root widget needs to be attached to a GTK container in the parent's `init_view` function.
    pub fn new(
        parent_model: &ParentModel,
        parent_widgets: &ParentModel::Widgets,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let mut widgets = Model::Widgets::init_view(&model, parent_widgets, sender.clone());
        let root_widget = widgets.root_widget();

        let components = Model::Components::init_components(&model, &widgets, sender.clone());
        let cloned_sender = sender.clone();

        widgets.connect_components(&components);

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                widgets.view(&model, sender.clone());
                glib::Continue(true)
            });
        }

        RelmComponent {
            widgets: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
            root_widget,
        }
    }

    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }

    pub fn root_widget(&self) -> &<Model::Widgets as WidgetsTrait<Model, ParentModel>>::Root {
        &self.root_widget
    }
}

impl<Model, ParentModel> RelmComponent<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel> + Send + 'static,
    Model::Widgets: WidgetsTrait<Model, ParentModel> + 'static,
    Model::Components: Send,
    Model::Msg: Send,
    ParentModel: ModelTrait,
    ParentModel::Msg: Send,
{
    pub fn with_new_thread(
        parent_model: &ParentModel,
        parent_widgets: &ParentModel::Widgets,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (global_sender, global_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let model = Model::init_model(parent_model);

        let mut widgets = Model::Widgets::init_view(&model, parent_widgets, sender.clone());
        let root_widget = widgets.root_widget();

        let components = Model::Components::init_components(&model, &widgets, sender.clone());
        let cloned_sender = sender.clone();

        widgets.connect_components(&components);

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
            receiver.attach(Some(&context), move |msg: Model::Msg| {
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
            parent_model: PhantomData,
            sender: cloned_sender,
            root_widget,
        }
    }
}
