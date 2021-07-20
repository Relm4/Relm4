use glib::Sender;

/// Widgets are part of an app or components. They represent the UI
/// that usually consists out of GTK widgets. The root represents the
/// widget that all other widgets are attached to.
/// The root of the main app must be a [`gtk::ApplicationWindow`].
pub trait RelmWidgets<Model, Components, Msg> {
    type Root: glib::IsA<gtk::Widget>;

    /// Initialize the UI.
    fn init_view(model: &Model, component: &Components, sender: glib::Sender<Msg>) -> Self;

    /// Return the root widget.
    fn root_widget(&self) -> Self::Root;
}

pub trait RelmComponents<ParentModel, ParentMsg> {
    fn init_components(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self;
}

/// Methods that initialize and update the main app.
pub trait AppUpdate<Components, Msg> {
    type Widgets;
    /// Update the model.
    fn update(&mut self, msg: Msg, components: &Components, sender: Sender<Msg>);

    /// Update the view to represent the updated model.
    fn view(&self, widgets: &mut Self::Widgets, sender: Sender<Msg>);
}

/// Methods that initialize and update a component.
pub trait ComponentUpdate<Components, Msg, ParentModel, ParentMsg> {
    type Widgets;

    fn init_model(parent_model: &ParentModel) -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    fn update(
        &mut self,
        msg: Msg,
        components: &Components,
        sender: Sender<Msg>,
        parent_sender: Sender<ParentMsg>,
    );

    /// Update the view to represent the updated model.
    fn view(&self, widgets: &mut Self::Widgets, sender: Sender<Msg>);
}

impl<ParentModel, ParentMsg> RelmComponents<ParentModel, ParentMsg> for () {
    fn init_components(_parent_model: &ParentModel, _sender: Sender<ParentMsg>) {}
}
