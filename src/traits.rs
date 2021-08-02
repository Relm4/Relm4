use gtk::glib::{self, Sender};
use gtk::prelude::StyleContextExt;

/// Widgets are part of an app or components. They represent the UI
/// that usually consists out of GTK widgets. The root represents the
/// widget that all other widgets are attached to.
/// The root of the main app must be a [`gtk::ApplicationWindow`].
pub trait RelmWidgets {
    type Root: glib::IsA<gtk::Widget>;
    type Model;
    type Components;
    type Msg;

    /// Initialize the UI.
    fn init_view(
        model: &Self::Model,
        component: &Self::Components,
        sender: Sender<Self::Msg>,
    ) -> Self;

    /// Return the root widget.
    fn root_widget(&self) -> Self::Root;

    /// Update the view to represent the updated model.
    fn view(&mut self, model: &Self::Model, sender: Sender<Self::Msg>);
}

pub trait RelmComponents<ParentModel, ParentMsg> {
    //type ParentModel;
    //type PargentMsg;

    fn init_components(parent_model: &ParentModel, parent_sender: Sender<ParentMsg>) -> Self;
}

/// Methods that initialize and update the main app.
pub trait AppUpdate {
    type Components;
    type Msg;
    /// Update the model.
    fn update(&mut self, msg: Self::Msg, components: &Self::Components, sender: Sender<Self::Msg>);
}

/// Methods that initialize and update a component.
pub trait ComponentUpdate {
    type Components;
    type Msg;
    type ParentModel;
    type ParentMsg;

    fn init_model(parent_model: &Self::ParentModel) -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: Sender<Self::Msg>,
        parent_sender: Sender<Self::ParentMsg>,
    );
}

#[cfg(feature = "tokio-rt")]
#[async_trait::async_trait]
pub trait AsyncComponentUpdate<Components, Msg, ParentModel, ParentMsg> {
    fn init_model(parent_model: &ParentModel) -> Self;

    /// Update the model. The parent_sender allows to send messages to the parent.
    async fn update(
        &mut self,
        msg: Msg,
        components: &Components,
        sender: Sender<Msg>,
        parent_sender: Sender<ParentMsg>,
    );
}

pub trait WidgetPlus {
    fn set_margin_all(&self, margin: i32);
    fn add_class_name(&self, class: &str);
    fn inline_css(&self, style_data: &[u8]);
}

impl<W: gtk::prelude::WidgetExt> WidgetPlus for W {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_start(margin);
        self.set_margin_end(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }

    fn add_class_name(&self, class: &str) {
        self.style_context().add_class(class);
    }

    fn inline_css(&self, style_data: &[u8]) {
        let context = self.style_context();
        let provider = gtk::CssProvider::new();
        provider.load_from_data(&[b"*{", style_data, b"}"].concat());
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }
}

impl<ParentModel, ParentMsg> RelmComponents<ParentModel, ParentMsg> for () {
    fn init_components(_parent_model: &ParentModel, _sender: Sender<ParentMsg>) {}
}
