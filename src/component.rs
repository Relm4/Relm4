use gtk::glib::{self, Sender};

use std::cell::{RefCell, RefMut};
use std::marker::{PhantomData, Send};
use std::rc::Rc;
use std::sync::mpsc::channel;

use crate::{ComponentUpdate, Components, Model as ModelTrait, Widgets as WidgetsTrait};

/// A component that can be part of the main application or other components.
///
/// A [`RelmComponent`] has its own widget, model and message type
/// and can send messages to its parent and its children components.
///
/// Multiple [`RelmComponent`]s that have the same parent are usually bundled in a struct that implements [`Components`].
#[derive(Debug)]
pub struct RelmComponent<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel> + 'static,
    ParentModel: ModelTrait,
    Model::Widgets: WidgetsTrait<Model, ParentModel> + 'static,
{
    model: PhantomData<Model>,
    parent_model: PhantomData<ParentModel>,
    sender: Sender<Model::Msg>,
    root_widget: <Model::Widgets as WidgetsTrait<Model, ParentModel>>::Root,
    shared_widgets: Option<Rc<RefCell<Model::Widgets>>>,
}

impl<Model, ParentModel> RelmComponent<Model, ParentModel>
where
    Model::Widgets: WidgetsTrait<Model, ParentModel> + 'static,
    ParentModel: ModelTrait,
    Model: ComponentUpdate<ParentModel> + 'static,
{
    /// Create a new [`RelmComponent`].
    pub fn new(
        parent_model: &ParentModel,
        parent_widgets: &ParentModel::Widgets,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let widgets = Model::Widgets::init_view(&model, parent_widgets, sender.clone());
        let root_widget = widgets.root_widget();

        let components = Model::Components::init_components(&model, &widgets, sender.clone());
        let cloned_sender = sender.clone();

        widgets.connect_components(&model, &components);

        let shared_widgets = Rc::new(RefCell::new(widgets));
        let handler_widgets = shared_widgets.clone();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                if let Ok(ref mut widgets) = handler_widgets.try_borrow_mut() {
                    widgets.view(&model, sender.clone());
                } else {
                    log::warn!("Could not mutably borrow the widgets. Make sure you drop all references to widgets after use");
                }
                glib::Continue(true)
            });
        }

        RelmComponent {
            model: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
            root_widget,
            shared_widgets: Some(shared_widgets),
        }
    }

    /// Send a message to this component.
    /// This can be used by the parent to send messages to this component.
    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }

    /// Returns the root widget of this component's widgets.
    /// Can be used by the parent [`Widgets::connect_components`](fn@crate::Widgets::connect_components) to connect the root widget
    /// to the parent's widgets.
    pub fn root_widget(&self) -> &<Model::Widgets as WidgetsTrait<Model, ParentModel>>::Root {
        &self.root_widget
    }

    /// Returns a mutable reference to the widgets of this component or [`None`] if
    /// you already have a refernce to the widgets.
    ///
    /// Use this carefully and make sure the reference to the widgets is dropped after use because
    /// otherwise the view function can't be called as long you own the widgets (it uses [`RefCell`] internally).
    pub fn widgets(&self) -> Option<RefMut<'_, Model::Widgets>> {
        self.shared_widgets
            .as_ref()
            .expect("Component wasn't initialized correctly: shared widgets are missing")
            .try_borrow_mut()
            .ok()
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
    /// Create a new [`RelmComponent`] that runs the [`ComponentUpdate::update`] function in another thread.
    ///
    /// Because GTK4 widgets are neither [`Send`] nor [`Sync`] we must still run the [`Widgets::view`](WidgetsTrait::view) function
    /// on the main thread.
    /// Also the model needs to be sent between threads to run the update and view functions.
    /// So if you look want to send a lot of messages to self using a [`RelmWorker`](crate::RelmWorker) will perform better.
    pub fn with_new_thread(
        parent_model: &ParentModel,
        parent_widgets: &ParentModel::Widgets,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (global_sender, global_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let model = Model::init_model(parent_model);

        let widgets = Model::Widgets::init_view(&model, parent_widgets, sender.clone());
        let root_widget = widgets.root_widget();

        let components = Model::Components::init_components(&model, &widgets, sender.clone());
        let cloned_sender = sender.clone();

        widgets.connect_components(&model, &components);

        let shared_widgets = Rc::new(RefCell::new(widgets));
        let handler_widgets = shared_widgets.clone();

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
            if let Ok(ref mut widgets) = handler_widgets.try_borrow_mut() {
                widgets.view(&model, view_sender.clone());
            } else {
                log::warn!("Could not mutably borrow the widgets. Make sure you drop all references to widgets after use");
            }
            model_tx.send(model).unwrap();
            glib::Continue(true)
        });

        RelmComponent {
            model: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
            root_widget,
            shared_widgets: Some(shared_widgets),
        }
    }
}
