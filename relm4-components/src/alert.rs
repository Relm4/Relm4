use gtk::prelude::{DialogExt, GtkWindowExt, WidgetExt};
use relm4::{send, ComponentUpdate, Model, Sender, WidgetPlus};

pub struct AlertSettings {
    pub text: String,
    pub secondary_text: Option<String>,
    pub is_modal: bool,
    pub destructive_accept: bool,
    pub confirm_label: String,
    pub cancel_label: String,
    pub option_label: Option<String>,
}

pub struct AlertModel {
    settings: AlertSettings,
    is_active: bool,
}

pub enum AlertMsg {
    Show,
    Response(gtk::ResponseType),
}

impl Model for AlertModel {
    type Msg = AlertMsg;
    type Widgets = AlertWidgets;
    type Components = ();
}

pub trait AlertParent: Model
where
    Self::Widgets: AlertParentWidgets,
{
    fn alert_config(&self) -> AlertSettings;
    fn confirm_msg() -> Self::Msg;
    fn cancel_msg() -> Self::Msg;
    fn option_msg() -> Self::Msg;
}

pub trait AlertParentWidgets {
    fn parent_window(&self) -> Option<gtk::Window>;
}

impl<ParentModel> ComponentUpdate<ParentModel> for AlertModel
where
    ParentModel: AlertParent,
    ParentModel::Widgets: AlertParentWidgets,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        AlertModel {
            settings: parent_model.alert_config(),
            is_active: false,
        }
    }

    fn update(
        &mut self,
        msg: AlertMsg,
        _components: &(),
        _sender: Sender<AlertMsg>,
        parent_sender: Sender<ParentModel::Msg>,
    ) {
        match msg {
            AlertMsg::Show => {
                self.is_active = true;
            }
            AlertMsg::Response(ty) => {
                self.is_active = false;
                parent_sender
                    .send(match ty {
                        gtk::ResponseType::Accept => ParentModel::confirm_msg(),
                        gtk::ResponseType::Other(_) => ParentModel::option_msg(),
                        _ => ParentModel::cancel_msg(),
                    })
                    .unwrap();
            }
        }
    }
}

#[relm4_macros::widget(pub)]
impl<ParentModel> relm4::Widgets<AlertModel, ParentModel> for AlertWidgets
where
    ParentModel: AlertParent,
    ParentModel::Widgets: AlertParentWidgets,
{
    view! {
        dialog = gtk::MessageDialog {
            set_modal: model.settings.is_modal,
            set_transient_for: parent_widgets.parent_window().as_ref(),
            set_text: Some(&model.settings.text),
            set_secondary_text: model.settings.secondary_text.as_deref(),
            set_message_type: gtk::MessageType::Question,
            set_visible: watch!(model.is_active),
            add_button: args!(&model.settings.confirm_label, gtk::ResponseType::Accept),
            add_button: args!(&model.settings.cancel_label, gtk::ResponseType::Cancel),
            connect_response(sender) => move |_, response| {
                send!(sender, AlertMsg::Response(response));
            }
        }
    }
    manual_init! {
        if let Some(option_label) = &model.settings.option_label {
            dialog.add_button(option_label, gtk::ResponseType::Other(0));
        }
        if model.settings.destructive_accept {
            let accept_widget = dialog.widget_for_response(gtk::ResponseType::Accept).expect("No button for accept response set");
            accept_widget.add_class_name("destructive-action");
        }
    }
}
