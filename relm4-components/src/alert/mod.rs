//! Reusable and easily configurable alert component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/alert.rs)**

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};

static LIBADWAITA_ENABLED: bool = cfg!(feature = "libadwaita");
static COMPONENT_CSS: &str = include_str!("style.css");

static ERROR_CSS: &str = "error";
static TITLE_CSS: &str = "title-2";
static DESTRUCTIVE_CSS: &str = "destructive-action";
static FLAT_CSS: &str = "flat";

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
    Response(AlertResponse),
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
        gtk::Window {
            #[watch]
            set_visible: model.is_active,
            add_css_class: "relm4-alert",

            #[wrap(Some)]
            set_titlebar = &gtk::Box {
                set_visible: false,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    set_margin_top: 16,
                    set_margin_bottom: 16,
                    set_margin_end: 30,
                    set_margin_start: 30,

                    gtk::Label {
                        set_text: &model.settings.text,
                        add_css_class: TITLE_CSS,
                    },

                    gtk::Label {
                        set_text: model.settings.secondary_text.as_deref().unwrap_or_default(),
                        set_justify: gtk::Justification::Center,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::End,
                    gtk::Separator {},

                    gtk::Box {
                        set_homogeneous: true,
                        set_vexpand: true,
                        set_valign: gtk::Align::End,

                        // The confirm widget is a bit more complicated than the rest, since we have destructive coloring on it sometimes.
                        //
                        // - On GTK, we want the *background* of the button to be red.
                        // - On Adwaita, we want the *text* of the button to be red.
                        #[name(confirm_label)]
                        gtk::Button {
                            #[watch]
                            set_visible: model.settings.confirm_label.is_some(),
                            #[watch]
                            set_class_active: (DESTRUCTIVE_CSS, !LIBADWAITA_ENABLED && model.settings.destructive_accept),
                            #[watch]
                            set_class_active: (FLAT_CSS, LIBADWAITA_ENABLED || !model.settings.destructive_accept),
                            set_hexpand: true,
                            connect_clicked => AlertMsg::Response(AlertResponse::Confirm),

                            gtk::Label {
                                #[watch]
                                set_label: model.settings.confirm_label.as_deref().unwrap_or_default(),
                                #[watch]
                                set_class_active: (ERROR_CSS, LIBADWAITA_ENABLED && model.settings.destructive_accept),
                            }
                        },

                        gtk::Box {
                            #[watch]
                            set_visible: model.settings.cancel_label.is_some(),

                            gtk::Separator {},

                            #[name(cancel_label)]
                            gtk::Button {
                                #[watch]
                                set_label: model.settings.cancel_label.as_deref().unwrap_or_default(),
                                add_css_class: FLAT_CSS,
                                set_hexpand: true,
                                connect_clicked => AlertMsg::Response(AlertResponse::Cancel)
                            }
                        },

                        gtk::Box {
                            #[watch]
                            set_visible: model.settings.option_label.is_some(),

                            gtk::Separator {},

                            #[name(option_label)]
                            gtk::Button {
                                #[watch]
                                set_label: model.settings.option_label.as_deref().unwrap_or_default(),
                                add_css_class: FLAT_CSS,
                                set_hexpand: true,
                                connect_clicked => AlertMsg::Response(AlertResponse::Option)
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        settings: AlertSettings,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        relm4::set_global_css_with_priority(
            COMPONENT_CSS,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

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
            AlertMsg::Show => self.is_active = true,
            AlertMsg::Hide => self.is_active = false,
            AlertMsg::Response(resp) => {
                self.is_active = false;
                sender.output(resp).unwrap();
            }
        }

        self.update_view(widgets, sender);
    }
}
