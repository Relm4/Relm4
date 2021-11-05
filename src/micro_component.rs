use gtk::glib::{self, Sender};

use std::cell::BorrowError;
use std::cell::BorrowMutError;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

pub enum MicroComponentError {
    Borrow(BorrowError),
    BorrowMut(BorrowMutError),
}

impl From<BorrowError> for MicroComponentError {
    fn from(err: BorrowError) -> Self {
        MicroComponentError::Borrow(err)
    }
}

impl From<BorrowMutError> for MicroComponentError {
    fn from(err: BorrowMutError) -> Self {
        MicroComponentError::BorrowMut(err)
    }
}

impl Display for MicroComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MicroComponentError::Borrow(err) => {
                f.write_fmt(format_args!("MicroComponentError::Borrow({})", err))
            },
            MicroComponentError::BorrowMut(err) => {
                f.write_fmt(format_args!("MicroComponentError::BorrowMut({})", err))
            }
        }
    }
}

impl Debug for MicroComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MicroComponentError::Borrow(err) => {
                f.debug_tuple("MicroComponentError::Borrow")
                    .field(err)
                    .finish()
            },
            MicroComponentError::BorrowMut(err) => {
                f.debug_tuple("MicroCompomponentError::BorrowMut")
                    .field(err)
                    .finish()
            }
        }
    }
}

pub struct MicroComponent<Model: MicroModel> {
    model: Rc<RefCell<Model>>,
    widgets: Rc<RefCell<Model::Widgets>>,
    root_widget: <Model::Widgets as MicroWidgets<Model>>::Root,
    data: Rc<RefCell<Model::Data>>, // for additional data such as senders to other components
    sender: Sender<Model::Msg>,
}


impl<Model> MicroComponent<Model> 
where
    Model::Widgets: MicroWidgets<Model> + 'static,
    Model::Msg: 'static,
    Model::Data: 'static,
    Model: MicroModel + 'static,
{
    pub fn new(model: Model, data: Model::Data) -> Self { 
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let widgets = Model::Widgets::init_view(&model, sender.clone());
        let root_widget = widgets.root_widget();
        
        let cloned_sender = sender.clone();
        let shared_widgets = Rc::new(RefCell::new(widgets));
        let handler_widgets = shared_widgets.clone();
        let shared_model = Rc::new(RefCell::new(model));
        let handler_model = shared_model.clone();
        let shared_data = Rc::new(RefCell::new(data));
        let handler_data = shared_data.clone();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                if let Ok(ref mut model) = handler_model.try_borrow_mut() {
                    if let Ok(ref data) = handler_data.try_borrow() {
                        model.update(msg, &data, sender.clone());
                        if let Ok(ref mut widgets) = handler_widgets.try_borrow_mut() {
                            widgets.view(&model, sender.clone());
                        } else {
                            log::warn!("Could not mutably borrow the widgets. Make sure you drop all references to widgets after use");
                        }
                    }
                    else {
                        log::warn!("Could not borrow the data. Make sure you drop all references to data after use");
                    }
                }
                else {
                    log::warn!("Could not mutably borrow the model. Make sure you drop all references to model after use")
                }

                glib::Continue(true)
            });
        }

        MicroComponent {
            model: shared_model,
            widgets: shared_widgets,
            root_widget,
            data: shared_data,
            sender: cloned_sender,
        }
    }
    pub fn update_view(&self) -> Result<(), MicroComponentError> { 
        let mut widgets = self.widgets()?;
        let model = self.model()?;

        widgets.view(&model, self.sender());

        Result::Ok( () )
    }


    pub fn model(&self) -> Result<Ref<'_, Model>, BorrowError> {
        self.model
            .as_ref()
            .try_borrow()
    }

    pub fn model_mut(&self) -> Result<RefMut<'_, Model>, BorrowMutError> { 
        self.model
            .as_ref()
            .try_borrow_mut()
    }
    pub fn widgets(&self) -> Result<RefMut<'_, Model::Widgets>, BorrowMutError> { 
        self.widgets
            .as_ref()
            .try_borrow_mut()
    }
    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> { 
        self.sender.send(msg)
    }

    pub fn sender(&self) -> Sender<Model::Msg> { 
        self.sender.clone()
    }

    pub fn root_widget(&self) -> &<Model::Widgets as MicroWidgets<Model>>::Root {
        &self.root_widget
    }
}

pub trait MicroModel {
    type Msg;
    type Widgets: MicroWidgets<Self>;
    type Data;
    
    fn update(&mut self, msg: Self::Msg, data: &Self::Data, sender: Sender<Self::Msg>);
}

pub trait MicroWidgets<Model: MicroModel + ?Sized> {
    type Root;
    
    fn init_view(model: &Model, sender: Sender<Model::Msg>) -> Self;
    fn view(&mut self, model: &Model, sender: Sender<Model::Msg>);
    fn root_widget(&self) -> Self::Root;
}