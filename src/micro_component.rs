use gtk::glib::{self, Sender};

use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;

pub struct MicroComponent<Model: MicroModel> {
    model: Option<Rc<RefCell<Model>>>,
    widgets: Option<Rc<RefCell<Model::Widgets>>>,
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
            model: Some(shared_model),
            widgets: Some(shared_widgets),
            root_widget,
            data: shared_data,
            sender: cloned_sender,
        }
    }
    pub fn update_view(&self) { 
        todo!()
    }


    pub fn model(&self) -> Option<Ref<'_, Model>> {
        self.model
            .as_ref()
            .expect("MicroComponent wasn't initialized correctly: model is missing")
            .try_borrow()
            .ok()
    }

    pub fn model_mut(&self) -> Option<RefMut<'_, Model>> { 
        self.model
            .as_ref()
            .expect("MicroComponent wasn't initialized correctly: model is missing")
            .try_borrow_mut()
            .ok()
    }
    pub fn widgets(&self) -> Option<RefMut<'_, Model::Widgets>> { 
        self.widgets
            .as_ref()
            .expect("MicroComponent wasn't initialized correctly: widgets are missing")
            .try_borrow_mut()
            .ok()
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