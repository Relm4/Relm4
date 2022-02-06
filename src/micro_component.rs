use gtk::glib::{self, Sender};
use gtk::prelude::{IsA, WidgetExt};

use std::{
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::WidgetPlus;

/// Errors which might get returned from [`MicroComponent::update_view`] method
///
/// Both [`std::fmt::Display`] and [`std::fmt::Debug`] are implemented for this
/// enum so if you just would like to show an error you don't need to do the
/// matching.
#[derive(Debug)]
pub enum MicroComponentError {
    /// Error returned if borrow failed
    Borrow(BorrowError),
    /// Error returned if borrowing mutably failed
    BorrowMut(BorrowMutError),
}

/// Helper to convert values of [`std::cell::BorrowError`] into [`MicroComponentError`]
impl From<BorrowError> for MicroComponentError {
    fn from(err: BorrowError) -> Self {
        MicroComponentError::Borrow(err)
    }
}

/// Helper to convert values of [`std::cell::BorrowMutError`] into [`MicroComponentError`]
impl From<BorrowMutError> for MicroComponentError {
    fn from(err: BorrowMutError) -> Self {
        MicroComponentError::BorrowMut(err)
    }
}

/// Formats [`MicroComponentError`] for empty format `{}`
///
/// This allows you to print errors without doing `matching` or `if let` statements
impl Display for MicroComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MicroComponentError::Borrow(err) => f.write_fmt(format_args!("{}", err)),
            MicroComponentError::BorrowMut(err) => f.write_fmt(format_args!("{}", err)),
        }
    }
}

/// MicroComponent is a small component that lives in their parents model, can be modified from their
/// parents model but at the same time have their own widgets and update function
#[derive(Debug)]
pub struct MicroComponent<Model: MicroModel> {
    model: Rc<RefCell<Model>>,
    widgets: Rc<RefCell<Model::Widgets>>,
    root_widget: <Model::Widgets as MicroWidgets<Model>>::Root,
    sender: Sender<Model::Msg>,
}

impl<Model> MicroComponent<Model>
where
    Model::Widgets: MicroWidgets<Model> + 'static,
    Model::Msg: 'static,
    Model::Data: 'static,
    Model: MicroModel + 'static,
{
    /// Creates new [`MicroComponent`]
    pub fn new(model: Model, data: Model::Data) -> Self {
        // Make sure GTK is initialized in case this was added to the model.
        gtk::init().unwrap();

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let widgets = Model::Widgets::init_view(&model, sender.clone());
        let root_widget = widgets.root_widget();

        let cloned_sender = sender.clone();
        let shared_widgets = Rc::new(RefCell::new(widgets));
        let handler_widgets = shared_widgets.clone();
        let shared_model = Rc::new(RefCell::new(model));
        let handler_model = shared_model.clone();

        {
            let context = glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context");
            // The main loop executes the closure as soon as it receives the message
            receiver.attach(Some(&context), move |msg: Model::Msg| {
                if let Ok(ref mut model) = handler_model.try_borrow_mut() {
                    model.update(msg, &data, sender.clone());
                    if let Ok(ref mut widgets) = handler_widgets.try_borrow_mut() {
                        widgets.view(model, sender.clone());
                    } else {
                        log::warn!("Could not mutably borrow the widgets. Make sure you drop all references to widgets after use");
                    }
                } else {
                    log::warn!("Could not mutably borrow the model. Make sure you drop all references to model after use")
                }

                glib::Continue(true)
            });
        }

        MicroComponent {
            model: shared_model,
            widgets: shared_widgets,
            root_widget,
            sender: cloned_sender,
        }
    }

    /// Updates a view of this [`MicroComponent`]
    pub fn update_view(&self) -> Result<(), MicroComponentError> {
        let mut widgets = self.widgets()?;
        let model = self.model()?;

        widgets.view(&model, self.sender());

        Result::Ok(())
    }

    /// Returns model for this [`MicroComponent`]
    ///
    /// Use this carefully and make sure reference is dropped. It's using [`RefCell`] internally.
    pub fn model(&self) -> Result<Ref<'_, Model>, BorrowError> {
        self.model.as_ref().try_borrow()
    }

    /// Returns mutable reference to model for this [`MicroComponent`]
    ///
    /// Use this carefully and make sure reference is dropped. It's using [`RefCell`] internally.
    /// If you don't drop the reference any call to [`MicroComponent::update_view`] will fail.
    pub fn model_mut(&self) -> Result<RefMut<'_, Model>, BorrowMutError> {
        self.model.as_ref().try_borrow_mut()
    }

    /// Returns a mutable reference to the widgets of this [`MicroComponent`] or will fail
    /// when you already have a reference to the widgets
    ///
    /// Use this carefully and make sure the reference to the widgets is dropped after use because
    /// otherwise the view function can't be called as long you own the widgets (it uses [`RefCell`] internally).
    pub fn widgets(&self) -> Result<RefMut<'_, Model::Widgets>, BorrowMutError> {
        self.widgets.as_ref().try_borrow_mut()
    }

    /// Send a message to this [`MicroComponent`].
    /// This can be used by the parent to send messages to this.
    pub fn send(&self, msg: Model::Msg) -> Result<(), std::sync::mpsc::SendError<Model::Msg>> {
        self.sender.send(msg)
    }

    /// Get a sender to send messages to this [`MicroComponent`].
    pub fn sender(&self) -> Sender<Model::Msg> {
        self.sender.clone()
    }

    /// Returns the root widget of this component's widgets.
    pub fn root_widget(&self) -> &<Model::Widgets as MicroWidgets<Model>>::Root {
        &self.root_widget
    }
}

impl<Model> MicroComponent<Model>
where
    Model::Widgets: MicroWidgets<Model> + 'static,
    Model::Msg: 'static,
    Model::Data: 'static,
    Model: MicroModel + 'static,
    <Model::Widgets as MicroWidgets<Model>>::Root: IsA<gtk::Widget>,
{
    /// Returns [`true`] of the root widget is connected to a parent widget.
    pub fn is_connected(&self) -> bool {
        self.root_widget.parent().is_some()
    }

    /// Tries to disconnect the root widget from its parent widget.
    ///
    /// Returns [`true`] of the root widget was disconnected from the parent widget
    /// and [`false`] if nothing was done.
    pub fn try_diconnect_root(&self) -> bool {
        if let Some(parent) = &self.root_widget.parent() {
            parent.try_remove(&self.root_widget)
        } else {
            false
        }
    }
}

/// Trait that defines the types associated with model used by [`MicroComponent`]
///
/// It can be anything that stores application state.
pub trait MicroModel {
    /// The message type that defines the messages that can be sent to modify the model.
    type Msg: 'static;

    /// The widgets type that can initialize and update the GUI with the data the model provides.
    ///
    /// If you don't want any widgets (for example for defining a worker), just use `()` here.
    type Widgets: MicroWidgets<Self> + Debug;

    /// Data that can be used to store senders and other stuff according to the needs of the user
    type Data;

    /// Updates the model.
    /// Typically a `match` statement is used to process the message.
    fn update(&mut self, msg: Self::Msg, data: &Self::Data, sender: Sender<Self::Msg>);
}

/// Define behavior to turn the data of your [`MicroModel`] into widgets.
pub trait MicroWidgets<Model: MicroModel + ?Sized> {
    /// The root represents the first widget that all other widgets of this [`MicroComponent`] are attached to.
    type Root: std::fmt::Debug;

    /// Initialize the UI.
    ///
    /// Use the sender to connect UI events and send messages back to modify the model.
    fn init_view(model: &Model, sender: Sender<Model::Msg>) -> Self;

    /// Update the view to represent the updated model.
    fn view(&mut self, model: &Model, sender: Sender<Model::Msg>);

    /// Return a clone of the root widget. This is typically a GTK4 widget.
    fn root_widget(&self) -> Self::Root;
}
