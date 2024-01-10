//! Reusable and easily configurable alert component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/alert.rs)**

use gtk::prelude::{ButtonExt, DialogExt, GtkWindowExt, WidgetExt};
use relm4::{gtk, Component, ComponentParts, ComponentSender};

static DESTRUCTIVE_CSS: &str = "destructive-action";

/// Configuration for the alert dialog component
///
/// The configuration object provides a [`Default`] implementation for any fields you don't want to manually specify, which is configured as such:
///
/// - `text` is set to "Alert".
/// - `secondary_text` is set to [`None`].
/// - `is_modal` is set to [`true`].
/// - `destructive_accept` is set to [`false`].
/// - `confirm_label` is set to [`None`].
/// - `cancel_label` is set to [`None`].
/// - `option_label` is set to [`None`].
#[derive(Debug)]
pub struct AlertSettings {
    /// Large text
    pub text: String,
    /// Optional secondary, smaller text
    pub secondary_text: Option<String>,
    /// Modal dialogs freeze other windows as long they are visible
    pub is_modal: bool,
    /// Sets color of the accept button to red if the theme supports it
    pub destructive_accept: bool,
    /// Text for confirm button. If [`None`] the button won't be shown.
    pub confirm_label: Option<String>,
    /// Text for cancel button. If [`None`] the button won't be shown.
    pub cancel_label: Option<String>,
    /// Text for third option button. If [`None`] the button won't be shown.
    pub option_label: Option<String>,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            text: String::from("Alert"),
            secondary_text: None,
            is_modal: true,
            destructive_accept: false,
            confirm_label: None,
            cancel_label: None,
            option_label: None,
        }
    }
}

/// Alert dialog component.
#[derive(Debug)]
pub struct Alert {
    /// The settings used by the alert component.
    pub settings: AlertSettings,
    is_active: bool,
}

/// Messages that can be sent to the alert dialog component
#[derive(Debug)]
pub enum AlertMsg {
    /// Message sent by the parent to view the dialog
    Show,

    /// Message sent by the parent to hide the dialog
    Hide,

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
impl Component for Alert {
    type Init = AlertSettings;
    type Input = AlertMsg;
    type Output = AlertResponse;
    type CommandOutput = ();

    view! {
        #[name(dialog)]
        gtk::MessageDialog {
            set_message_type: gtk::MessageType::Question,
            #[watch]
            set_visible: model.is_active,
            connect_response[sender] => move |_, response| {
                sender.input(AlertMsg::Response(response));
            },

            // Apply configuration
            #[watch]
            set_text: Some(&model.settings.text),
            #[watch]
            set_secondary_text: model.settings.secondary_text.as_deref(),
            #[watch]
            set_modal: model.settings.is_modal,

            #[name(accept_widget)]
            add_action_widget[gtk::ResponseType::Accept] = &gtk::Button {
                #[watch]
                set_label: model.settings.confirm_label.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.settings.confirm_label.is_some(),
            },

            add_action_widget[gtk::ResponseType::Cancel] = &gtk::Button {
                #[watch]
                set_label: model.settings.cancel_label.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.settings.cancel_label.is_some()
            },

            add_action_widget[gtk::ResponseType::Other(0)] = &gtk::Button {
                #[watch]
                set_label: model.settings.option_label.as_deref().unwrap_or(""),
                #[watch]
                set_visible: model.settings.option_label.is_some()
            }
        }
    }

    fn init(
        settings: AlertSettings,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Alert {
            settings,
            is_active: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        input: AlertMsg,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match input {
            AlertMsg::Show => {
                self.is_active = true;
            }
            AlertMsg::Hide => {
                self.is_active = false;
            }
            AlertMsg::Response(ty) => {
                self.is_active = false;
                sender
                    .output(match ty {
                        gtk::ResponseType::Accept => AlertResponse::Confirm,
                        gtk::ResponseType::Other(_) => AlertResponse::Option,
                        _ => AlertResponse::Cancel,
                    })
                    .unwrap();
            }
        }

        if self.settings.destructive_accept {
            widgets.accept_widget.add_css_class(DESTRUCTIVE_CSS);
        } else {
            widgets.accept_widget.remove_css_class(DESTRUCTIVE_CSS);
        }

        self.update_view(widgets, sender);
    }
}
