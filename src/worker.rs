use gtk::glib::{self, Sender};

use std::marker::{PhantomData, Send};

use crate::{ComponentUpdate, Components, Model as ModelTrait};

#[derive(Clone)]
/// [`RelmWorker`]s are like [`RelmComponent`](crate::RelmComponent)s but they don't have any widgets.
///
/// They are usually used to run expansive tasks on different threads and report back when they are finished
/// so that their parent components can keep handling UI events in the meantime.
/// For example you could use a [`RelmWorker`] for sending a HTTP request or for copying files.
///
/// A [`RelmWorker`] has its own model and message type
/// and can send messages to its parent and its children components.
///
/// Multiple [`RelmWorker`]s that have the same parent are usually bundled along with [`RelmComponent`](crate::RelmComponent)s
/// in a struct that implements [`Components`].
pub struct RelmWorker<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel, Widgets = ()> + 'static,
    ParentModel: ModelTrait,
{
    model: PhantomData<Model>,
    parent_model: PhantomData<ParentModel>,
    sender: Sender<Model::Msg>,
}

impl<Model, ParentModel> RelmWorker<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel, Widgets = ()> + 'static,
    ParentModel: ModelTrait,
{
    /// Create a new [`RelmWorker`] that runs on the main thread.
    pub fn new(parent_model: &ParentModel, parent_sender: Sender<ParentModel::Msg>) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, &(), sender.clone());
        let cloned_sender = sender.clone();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                glib::Continue(true)
            });
        }

        RelmWorker {
            model: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
        }
    }

    /// Send a message to this component.
    /// This can be used by the parent to send messages to this worker.
    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }
}

impl<Model, ParentModel> RelmWorker<Model, ParentModel>
where
    Model: ComponentUpdate<ParentModel, Widgets = ()> + Send + 'static,
    Model::Components: Send + 'static,
    Model::Msg: Send,
    ParentModel: ModelTrait,
    ParentModel::Msg: Send,
{
    /// Create a new [`RelmWorker`] that runs the [`ComponentUpdate::update`] function in another thread.
    pub fn with_new_thread(
        parent_model: &ParentModel,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, &(), sender.clone());
        let cloned_sender = sender.clone();

        std::thread::spawn(move || {
            let context = glib::MainContext::new();
            context.push_thread_default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                glib::Continue(true)
            });

            let main_loop = glib::MainLoop::new(Some(&context), true);
            main_loop.run();
            context.pop_thread_default();
        });

        RelmWorker {
            model: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
        }
    }
}

#[cfg(feature = "tokio-rt")]
use crate::traits::AsyncComponentUpdate;

#[cfg(feature = "tokio-rt")]
#[derive(Clone)]
/// A [`AsyncRelmWorker`] is like a [`RelmWorker`] but runs the [`AsyncComponentUpdate::update`] function in
/// a tokio [`Runtime`](tokio::runtime::Runtime).
pub struct AsyncRelmWorker<Model, ParentModel>
where
    Model: AsyncComponentUpdate<ParentModel, Widgets = ()> + Send + 'static,
    Model::Components: Send,
    ParentModel: ModelTrait,
    ParentModel::Msg: Send,
    Model::Msg: Send,
{
    model: PhantomData<Model>,
    parent_model: PhantomData<ParentModel>,
    sender: Sender<Model::Msg>,
}

#[cfg(feature = "tokio-rt")]
impl<Model, ParentModel> AsyncRelmWorker<Model, ParentModel>
where
    Model: AsyncComponentUpdate<ParentModel, Widgets = ()> + Send,
    Model::Components: Send,
    ParentModel: ModelTrait,
    ParentModel::Msg: Send,
    Model::Msg: Send,
{
    /// Create a new [`AsyncRelmWorker`] that runs the [`AsyncComponentUpdate::update`] function in
    /// a tokio [`Runtime`](tokio::runtime::Runtime).
    pub fn with_new_tokio_rt(
        parent_model: &ParentModel,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, &(), sender.clone());
        let cloned_sender = sender.clone();

        std::thread::spawn(move || {
            let context = glib::MainContext::new();
            context.push_thread_default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                rt.block_on(model.update(msg, &components, sender.clone(), parent_sender.clone()));
                glib::Continue(true)
            });

            let main_loop = glib::MainLoop::new(Some(&context), true);
            main_loop.run();
            context.pop_thread_default();
        });

        AsyncRelmWorker {
            model: PhantomData,
            parent_model: PhantomData,
            sender: cloned_sender,
        }
    }

    /// Send a message to this component.
    /// This can be used by the parent to send messages to this worker.
    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }
}
