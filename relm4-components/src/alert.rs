//! Reusable and easily configurable alert component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/alert.rs)**

use gtk::prelude::{DialogExt, GtkWindowExt, WidgetExt};
use relm4::{gtk, send, ComponentParts, ComponentSender, SimpleComponent};

/// Configuration for the alert dialog component
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

/// Alert dialog component.
pub struct Alert {
    settings: AlertSettings,
    is_active: bool,
}

/// Messages that can be sent to the alert dialog component
pub enum AlertMsg {
    /// Message sent by the parent to view the dialog
    Show,

    #[doc(hidden)]
    Response(gtk::ResponseType),
}

/// User action performed on the alert dialog.
#[derive(Debug)]
pub enum AlertResponse {
    /// User clicked confirm button.
    Confirm,

    /// User clicked cancel button.
    Cancel,

    /// User clicked user-supplied option.
    Option,
}

/// Widgets of the alert dialog component.
#[relm4::component(pub)]
impl SimpleComponent for Alert {
    type Widgets = AlertWidgets;
    type InitParams = AlertSettings;
    type Input = AlertMsg;
    type Output = AlertResponse;

    view! {
        dialog = gtk::MessageDialog {
            set_message_type: gtk::MessageType::Question,
            set_visible: watch!(model.is_active),
            connect_response(sender) => move |_, response| {
                send!(sender.input, AlertMsg::Response(response));
            },

            // Apply configuration
            set_text: Some(&model.settings.text),
            set_secondary_text: model.settings.secondary_text.as_deref(),
            set_modal: model.settings.is_modal,
            add_button: args!(&model.settings.confirm_label, gtk::ResponseType::Accept),
            add_button: args!(&model.settings.cancel_label, gtk::ResponseType::Cancel),
        }
    }

    fn init(
        settings: AlertSettings,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Alert {
            settings,
            is_active: false,
        };

        let widgets = view_output!();

        if let Some(option_label) = &model.settings.option_label {
            widgets
                .dialog
                .add_button(option_label, gtk::ResponseType::Other(0));
        }

        if model.settings.destructive_accept {
            let accept_widget = widgets
                .dialog
                .widget_for_response(gtk::ResponseType::Accept)
                .expect("No button for accept response set");
            accept_widget.add_css_class("destructive-action");
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: AlertMsg, sender: &ComponentSender<Self>) {
        match input {
            AlertMsg::Show => {
                self.is_active = true;
            }
            AlertMsg::Response(ty) => {
                self.is_active = false;
                send!(
                    sender.output,
                    match ty {
                        gtk::ResponseType::Accept => AlertResponse::Confirm,
                        gtk::ResponseType::Other(_) => AlertResponse::Option,
                        _ => AlertResponse::Cancel,
                    }
                );
            }
        }
    }
}
