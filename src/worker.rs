use gtk::glib::{self, Sender};

use std::marker::{PhantomData, Send};

use crate::{ComponentUpdate, RelmComponents};

#[derive(Clone)]
pub struct RelmWorker<Model, Components, Msg, ParentModel, ParentMsg>
where
    Model: ComponentUpdate<Components, Msg, ParentModel, ParentMsg>,
{
    model: PhantomData<Model>,
    parent_info: PhantomData<(ParentModel, ParentMsg)>,
    components: PhantomData<Components>,
    sender: Sender<Msg>,
}

impl<Model, Components, Msg, ParentModel, ParentMsg>
    RelmWorker<Model, Components, Msg, ParentModel, ParentMsg>
where
    Model: ComponentUpdate<Components, Msg, ParentModel, ParentMsg> + 'static,
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

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                glib::Continue(true)
            });
        }

        RelmWorker {
            model: PhantomData,
            parent_info: PhantomData,
            components: PhantomData,
            sender: cloned_sender,
        }
    }

    pub fn send(&self, msg: Msg) -> Result<(), std::sync::mpsc::SendError<Msg>> {
        self.sender.send(msg)
    }
}

impl<Model, Components, Msg, ParentModel, ParentMsg>
    RelmWorker<Model, Components, Msg, ParentModel, ParentMsg>
where
    Model: ComponentUpdate<Components, Msg, ParentModel, ParentMsg> + Send + 'static,
    Components: RelmComponents<Model, Msg> + Send + 'static,
    ParentMsg: Send + 'static,
    Msg: Send + 'static,
{
    pub fn with_new_thread(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Components::init_components(&model, sender.clone());
        let cloned_sender = sender.clone();

        std::thread::spawn(move || {
            let context = glib::MainContext::new();
            context.push_thread_default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");

            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Msg| {
                model.update(msg, &components, sender.clone(), parent_sender.clone());
                glib::Continue(true)
            });

            let main_loop = glib::MainLoop::new(Some(&context), true);
            main_loop.run();
            context.pop_thread_default();
        });

        RelmWorker {
            model: PhantomData,
            parent_info: PhantomData,
            components: PhantomData,
            sender: cloned_sender,
        }
    }
}

#[cfg(feature = "tokio-rt")]
#[derive(Clone)]
pub struct AsyncRelmWorker<Model, Components, Msg, ParentModel, ParentMsg>
where
    Model: crate::traits::AsyncComponentUpdate<Components, Msg, ParentModel, ParentMsg>,
{
    model: PhantomData<Model>,
    parent_info: PhantomData<(ParentModel, ParentMsg)>,
    components: PhantomData<Components>,
    sender: Sender<Msg>,
}

#[cfg(feature = "tokio-rt")]
impl<Model, Components, Msg, ParentModel, ParentMsg>
    AsyncRelmWorker<Model, Components, Msg, ParentModel, ParentMsg>
where
    Model: crate::traits::AsyncComponentUpdate<Components, Msg, ParentModel, ParentMsg>
        + Send
        + 'static,
    Components: RelmComponents<Model, Msg> + Send + 'static,
    ParentMsg: Send + 'static,
    Msg: Send + 'static,
{
    pub fn with_new_tokio_rt(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut model = Model::init_model(parent_model);

        let components = Components::init_components(&model, sender.clone());
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
            receiver.attach(Some(&context), move |msg: Msg| {
                rt.block_on(model.update(msg, &components, sender.clone(), parent_sender.clone()));
                glib::Continue(true)
            });

            let main_loop = glib::MainLoop::new(Some(&context), true);
            main_loop.run();
            context.pop_thread_default();
        });

        AsyncRelmWorker {
            model: PhantomData,
            parent_info: PhantomData,
            components: PhantomData,
            sender: cloned_sender,
        }
    }

    pub fn send(&self, msg: Msg) -> Result<(), std::sync::mpsc::SendError<Msg>> {
        self.sender.send(msg)
    }
}
