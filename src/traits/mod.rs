use gtk::glib::Sender;

mod impls;

pub trait Model: std::marker::Sized {
    type Msg: 'static;
    type Widgets;
    type Components: Components<Self>;
}

/// Methods that initialize and update the main app.
pub trait AppUpdate: Model {
    /// Update the model.
    fn update(&mut self, msg: Self::Msg, components: &Self::Components, sender: Sender<Self::Msg>);
}

/// Methods that initialize and update a component.
pub trait ComponentUpdate<ParentModel: Model>: Model {
    fn init_model(parent_model: &ParentModel) -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: Sender<Self::Msg>,
        parent_sender: Sender<ParentModel::Msg>,
    );
}

/// Widgets are part of an app or components. They represent the UI
/// that usually consists out of GTK widgets. The root represents the
/// widget that all other widgets are attached to.
/// The root of the main app must be a [`gtk::ApplicationWindow`].
pub trait Widgets<ModelType, ParentModel>
where
    ModelType: Model<Widgets = Self>,
    ParentModel: Model,
{
    type Root;

    /// Initialize the UI.
    fn init_view(
        model: &ModelType,
        parent_widgets: &ParentModel::Widgets,
        sender: Sender<ModelType::Msg>,
    ) -> Self;

    fn connect_components(&self, _components: &ModelType::Components) {}

    /// Return the root widget.
    fn root_widget(&self) -> Self::Root;

    /// Update the view to represent the updated model.
    fn view(&mut self, model: &ModelType, sender: Sender<ModelType::Msg>);
}

pub trait Components<ParentModel: ?Sized + Model> {
    fn init_components(
        parent_model: &ParentModel,
        parent_widget: &ParentModel::Widgets,
        parent_sender: Sender<ParentModel::Msg>,
    ) -> Self;
}

#[cfg(feature = "tokio-rt")]
#[async_trait::async_trait]
pub trait AsyncComponentUpdate<ParentModel: Model>: Model {
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
