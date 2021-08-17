//! Reusable and easily configurable alert component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/alert.rs)**

use gtk::prelude::{DialogExt, GtkWindowExt, WidgetExt};
use relm4::{send, ComponentUpdate, Model, Sender, WidgetPlus};

pub struct AlertSettings {
    /// Large text
    pub text: String,
    /// Optional secondary, smaller text
    pub secondary_text: Option<String>,
    /// Modal dialogs freeze other windows as long they are visible
    pub is_modal: bool,
    /// Sets color of the accept button to red if the theme supports it
    pub destructive_accept: bool,
    /// Text for confirm button
    pub confirm_label: String,
    /// Text for cancel button
    pub cancel_label: String,
    /// Text for third option button. If [`None`] the third button won't be created.
    pub option_label: Option<String>,
}

pub struct AlertModel {
    settings: AlertSettings,
    is_active: bool,
}

pub enum AlertMsg {
    /// Message sent by the parent to view the dialog
    Show,
    #[doc(hidden)]
    Response(gtk::ResponseType),
}

impl Model for AlertModel {
    type Msg = AlertMsg;
    type Widgets = AlertWidgets;
    type Components = ();
}

/// Interface for the parent model
pub trait AlertParent: Model
where
    Self::Widgets: AlertParentWidgets,
{
    /// Configuration for alert component.
    fn alert_config(&self) -> AlertSettings;

    /// Message sent to parent if user clicks confirm button
    fn confirm_msg() -> Self::Msg;

    /// Message sent to parent if user clicks cancel button
    fn cancel_msg() -> Self::Msg;

    /// Message sent to parent if user clicks third option button
    fn option_msg() -> Self::Msg;
}

/// Get the parent window that allows setting the parent window of the dialog with
/// [`gtk::prelude::GtkWindowExt::set_transient_for`].
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
            set_transient_for: parent_widgets.parent_window().as_ref(),
            set_message_type: gtk::MessageType::Question,
            set_visible: watch!(model.is_active),
            connect_response(sender) => move |_, response| {
                send!(sender, AlertMsg::Response(response));
            },

            // Apply configuration
            set_text: Some(&model.settings.text),
            set_secondary_text: model.settings.secondary_text.as_deref(),
            set_modal: model.settings.is_modal,
            add_button: args!(&model.settings.confirm_label, gtk::ResponseType::Accept),
            add_button: args!(&model.settings.cancel_label, gtk::ResponseType::Cancel),
        }
    }

    fn post_init() {
        if let Some(option_label) = &model.settings.option_label {
            dialog.add_button(option_label, gtk::ResponseType::Other(0));
        }
        if model.settings.destructive_accept {
            let accept_widget = dialog
                .widget_for_response(gtk::ResponseType::Accept)
                .expect("No button for accept response set");
            accept_widget.add_class_name("destructive-action");
        }
    }
}
