use gtk::glib::Sender;

mod impls;

/// Trait that defines the types associated with the model.
///
/// A model can be anything that stores application state.
///
/// # Example
///
/// ```
/// # use relm4::Model;
/// # struct AppWidgets {}
/// #
/// struct AppModel {
///     counter: u8,
/// }
///
/// enum AppMsg {
///     Increment,
///     Decrement,
/// }
///
/// impl Model for AppModel {
///     type Msg = AppMsg;
///     type Widgets = AppWidgets;
///     type Components = ();
/// }
/// ```
pub trait Model: std::marker::Sized {
    /// The message type that defines the messages that can be sent to modify the model.
    type Msg: 'static;

    /// The widgets type that can initialize and update the GUI with the data the model provides.
    ///
    /// If you don't want any widgets (for example for defining a worker), just use `()` here.
    type Widgets;

    /// The components type that initializes the child components of this model.
    ///
    /// If you don't want any component associated with this model just use `()`.
    type Components: Components<Self>;
}

/// Define the behavior to update the model of the main app.
///
/// # Example
///
/// ```
/// # use relm4::{AppUpdate, Sender, Model};
/// # struct AppWidgets {}
/// #
/// # struct AppModel {
/// #     counter: u8,
/// # }
/// #
/// # enum AppMsg {
/// #     Increment,
/// #     Decrement,
/// # }
/// #
/// # impl Model for AppModel {
/// #     type Msg = AppMsg;
/// #     type Widgets = AppWidgets;
/// #     type Components = ();
/// # }
/// #
/// impl AppUpdate for AppModel {
///     fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
///         match msg {
///             AppMsg::Increment => {
///                 self.counter += 1;
///             }
///             AppMsg::Decrement => {
///                 self.counter -= 1;
///             }
///         }
///         true
///     }
/// }
/// ```
pub trait AppUpdate: Model {
    /// Updates the model.
    /// Typically a `match` statement is used to process the message.
    ///
    /// Return [`true`] to continue running the application and return [`false`] to quit.
    ///
    /// Components and sender don't need to be used but help you sending messages to
    /// your components or queuing messages for self.
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: Sender<Self::Msg>,
    ) -> bool;
}

/// Define the behavior to initialize and update a component or worker.
pub trait ComponentUpdate<ParentModel: Model>: Model {
    /// Initialize the model of the component or worker.
    ///
    /// In case you want to share information or settings with the parent component you
    /// get the parent's model passed as parameter.
    fn init_model(parent_model: &ParentModel) -> Self;

    /// Updates the model.
    /// Typically a `match` statement is used to process the message.
    ///
    /// Components and sender don't need to be used but help you sending messages to
    /// your components or queuing messages for self.
    ///
    /// The parent sender allows to send messages to the parent component which for also can be the main app.
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: Sender<Self::Msg>,
        parent_sender: Sender<ParentModel::Msg>,
    );
}

/// A message handler that can be used in situations where a [`RelmWorker`](crate::RelmWorker)
/// isn't flexible enough.
pub trait MessageHandler<ParentModel: Model> {
    /// The message type of this message handler.
    type Msg;

    /// The sender type that can be used to send a message to a [`RelmMsgHandler`](crate::RelmMsgHandler).
    type Sender;

    /// Initialize this message handler.
    fn init(parent_model: &ParentModel, parent_sender: Sender<ParentModel::Msg>) -> Self;

    /// Sends a message to the message handler.
    fn send(&self, msg: Self::Msg);

    /// Get a sender for sending messages to this [`RelmMsgHandler`](crate::RelmMsgHandler).
    fn sender(&self) -> Self::Sender;
}

/// Define behavior to turn the data of you model into widgets.
///
/// This trait and the associated struct can also be implemented by the `relm4-macros::widget` macro.
///
/// This trait has two generic types, its own model and the model of the parent (which can be `()`).
/// This allows you to define widgets that can work with different models and parent models.
/// Most commonly this is used to create reusable components.
pub trait Widgets<ModelType, ParentModel>
where
    ModelType: Model<Widgets = Self>,
    ParentModel: Model,
{
    /// The root represents the first widget that all other widgets of this app or component are attached to.
    /// The root of the main app must be a [`gtk::ApplicationWindow`].
    type Root: std::fmt::Debug;

    /// Initialize the UI.
    ///
    /// Use the parent widgets to connect them to the widgets of this model.
    ///
    /// Use the sender to connect UI events and send messages back to modify the model.
    fn init_view(
        model: &ModelType,
        _components: &ModelType::Components,
        sender: Sender<ModelType::Msg>,
    ) -> Self;

    /// Optional method to initialize components.
    /// This is only useful if you want to attach the widgets of a component to the widgets of this model.
    fn connect_parent(&self, _parent_widgets: &ParentModel::Widgets) {}

    /// Return a clone of the root widget. This is typically a GTK4 widget.
    fn root_widget(&self) -> Self::Root;

    /// Update the view to represent the updated model.
    fn view(&mut self, model: &ModelType, sender: Sender<ModelType::Msg>);
}

/// Define how to initialize one or more components.
///
/// Typically a struct is used to store multiple components that are child
/// components of the app or another component.
///
/// # Example
///
/// ```
/// # use relm4::{RelmComponent, Components, AppUpdate, Sender, Model, ComponentUpdate};
/// # struct AppWidgets {}
/// #
/// # struct AppModel {
/// #     counter: u8,
/// # }
/// #
/// # enum AppMsg {
/// #     Increment,
/// #     Decrement,
/// # }
/// #
/// # impl Model for AppModel {
/// #     type Msg = AppMsg;
/// #     type Widgets = AppWidgets;
/// #     type Components = ();
/// # }
/// #
/// # struct CompMsg {};
/// # struct Comp1Model {};
/// # struct Comp2Model {};
/// #
/// # impl Model for Comp1Model {
/// #     type Msg = CompMsg;
/// #     type Widgets = ();
/// #     type Components = ();
/// # }
/// #
/// # impl Model for Comp2Model {
/// #     type Msg = CompMsg;
/// #     type Widgets = ();
/// #     type Components = ();
/// # }
/// #
/// # impl ComponentUpdate<AppModel> for Comp1Model {
/// #     fn init_model(_parent_model: &AppModel) -> Self {
/// #         Comp1Model {}
/// #     }
/// #
/// #     fn update(
/// #         &mut self,
/// #         message: CompMsg,
/// #         _components: &(),
/// #         _sender: Sender<CompMsg>,
/// #         _parent_sender: Sender<AppMsg>,
/// #     ) {}
/// # }
/// #
/// # impl ComponentUpdate<AppModel> for Comp2Model {
/// #     fn init_model(_parent_model: &AppModel) -> Self {
/// #         Comp2Model {}
/// #     }
/// #
/// #     fn update(
/// #         &mut self,
/// #         message: CompMsg,
/// #         _components: &(),
/// #         _sender: Sender<CompMsg>,
/// #         _parent_sender: Sender<AppMsg>,
/// #     ) {}
/// # }
/// #
/// struct AppComponents {
///     comp1: RelmComponent<Comp1Model, AppModel>,
///     comp2: RelmComponent<Comp2Model, AppModel>,
/// }
///
/// impl Components<AppModel> for AppComponents {
///     fn init_components(parent_model: &AppModel, parent_widgets: &AppWidgets, parent_sender: Sender<AppMsg>) -> Self {
///         AppComponents {
///             comp1: RelmComponent::with_new_thread(parent_model, parent_widgets, parent_sender.clone()),
///             comp2: RelmComponent::new(parent_model, parent_widgets, parent_sender),
///         }
///     }
/// }
/// ```
pub trait Components<ParentModel: ?Sized + Model> {
    /// Initialize your components and workers inside this function.
    fn init_components(parent_model: &ParentModel, parent_sender: Sender<ParentModel::Msg>)
        -> Self;

    /// Connect the components to their parent components widgets (to set the parent window for example).
    fn connect_parent(&mut self, _parent_widget: &ParentModel::Widgets);
}

#[cfg(feature = "tokio-rt")]
#[cfg_attr(doc, doc(cfg(feature = "tokio-rt")))]
#[async_trait::async_trait]
/// [`ComponentUpdate`] for asynchronous workers and components.
pub trait AsyncComponentUpdate<ParentModel: Model>: Model {
    /// Initialize the model of the component or worker.
    ///
    /// In case you want to share information or settings with the parent component you
    /// get the parent's model passed as parameter.
    fn init_model(parent_model: &ParentModel) -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    async fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: Sender<Self::Msg>,
        parent_sender: Sender<ParentModel::Msg>,
    );
}
