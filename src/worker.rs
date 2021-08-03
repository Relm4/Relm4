use gtk::glib::{self, Sender};

use std::marker::{PhantomData, Send};

use crate::{ComponentUpdate, Model as ModelTrait, RelmComponents};

#[derive(Clone)]
pub struct RelmWorker<Model>
where
    Model: ComponentUpdate,
{
    model: PhantomData<Model>,
    sender: Sender<Model::Msg>,
}

impl<Model> RelmWorker<Model>
where
    Model: ComponentUpdate + 'static,
{
    /// Create component. Usually you can store Self in the widgets of the parent component.
    /// The root widget needs to be attached to a GTK container in the parent's `init_view` function.
    pub fn new(
        parent_model: &Model::ParentModel,
        parent_sender: Sender<<Model::ParentModel as ModelTrait>::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, sender.clone());
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
            sender: cloned_sender,
        }
    }

    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }
}

impl<Model> RelmWorker<Model>
where
    Model: ComponentUpdate + Send + 'static,
    Model::Components: Send + 'static,
    Model::Msg: Send,
    <Model::ParentModel as ModelTrait>::Msg: Send,
{
    pub fn with_new_thread(
        parent_model: &Model::ParentModel,
        parent_sender: Sender<<Model::ParentModel as ModelTrait>::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, sender.clone());
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
            sender: cloned_sender,
        }
    }
}

#[cfg(feature = "tokio-rt")]
#[derive(Clone)]
pub struct AsyncRelmWorker<Model>
where
    Model: crate::traits::AsyncComponentUpdate,
{
    model: PhantomData<Model>,
    sender: Sender<Model::Msg>,
}

#[cfg(feature = "tokio-rt")]
impl<Model> AsyncRelmWorker<Model>
where
    Model: crate::traits::AsyncComponentUpdate + Send + 'static,
    Model::Components: Send + 'static,
    <Model::ParentModel as ModelTrait>::Msg: Send + 'static,
    Model::Msg: Send + 'static,
{
    pub fn with_new_tokio_rt(
        parent_model: &Model::ParentModel,
        parent_sender: Sender<<Model::ParentModel as ModelTrait>::Msg>,
    ) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Model::Components::init_components(&model, sender.clone());
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
            sender: cloned_sender,
        }
    }

    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }
}
