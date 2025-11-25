//! Reusable and easily configurable alert component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/alert.rs)**

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, gtk};
use std::sync::LazyLock;

const LIBADWAITA_ENABLED: bool = cfg!(feature = "libadwaita");
const COMPONENT_CSS: &str = include_str!("style.css");
const MESSAGE_AREA_CSS: &str = "message-area";
const RESPONSE_BUTTONS_CSS: &str = "response-buttons";

/// The initializer for the CSS, ensuring it only happens once.
static INITIALIZE_CSS: LazyLock<()> = LazyLock::new(|| {
    relm4::set_global_css_with_priority(COMPONENT_CSS, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
});

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
/// - `extra_child` is set to [`None`].
#[derive(Debug)]
pub struct AlertSettings {
    /// Large text
    pub text: Option<String>,
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
    /// An optional, extra widget to display below the secondary text.
    pub extra_child: Option<gtk::Widget>,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            text: Some("Alert".into()),
            secondary_text: None,
            is_modal: true,
            destructive_accept: false,
            confirm_label: None,
            cancel_label: None,
            option_label: None,
            extra_child: None,
        }
    }
}

/// Alert dialog component.
#[derive(Debug)]
pub struct Alert {
    /// The settings used by the alert component.
    pub settings: AlertSettings,
    is_active: bool,
    current_child: Option<gtk::Widget>,
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
            set_modal: model.settings.is_modal,
            add_css_class: "relm4-alert",

            #[wrap(Some)]
            set_titlebar = &gtk::Box {
                set_visible: false,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name(message_area)]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_vexpand: true,
                    add_css_class: MESSAGE_AREA_CSS,

                    gtk::Label {
                        #[watch]
                        set_text: model.settings.text.as_deref().unwrap_or_default(),
                        #[watch]
                        set_visible: model.settings.text.is_some(),
                        set_valign: gtk::Align::Start,
                        set_justify: gtk::Justification::Center,
                        add_css_class: relm4::css::TITLE_2,
                        set_wrap: true,
                        set_max_width_chars: 20,
                    },

                    gtk::Label {
                        #[watch]
                        set_text: model.settings.secondary_text.as_deref().unwrap_or_default(),
                        set_vexpand: true,
                        set_valign: gtk::Align::Fill,
                        set_justify: gtk::Justification::Center,
                        set_wrap: true,
                        set_max_width_chars: 40,
                    },
                },

                gtk::Box {
                    add_css_class: RESPONSE_BUTTONS_CSS,
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand_set: true,
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
                            set_class_active: (relm4::css::DESTRUCTIVE_ACTION, !LIBADWAITA_ENABLED && model.settings.destructive_accept),
                            #[watch]
                            set_class_active: (relm4::css::FLAT, LIBADWAITA_ENABLED || !model.settings.destructive_accept),
                            set_hexpand: true,
                            connect_clicked => AlertMsg::Response(AlertResponse::Confirm),

                            gtk::Label {
                                #[watch]
                                set_label: model.settings.confirm_label.as_deref().unwrap_or_default(),
                                #[watch]
                                set_class_active: (relm4::css::ERROR, LIBADWAITA_ENABLED && model.settings.destructive_accept),
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
                                add_css_class: relm4::css::FLAT,
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
                                add_css_class: relm4::css::FLAT,
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
        // Initialize the CSS.
        #[allow(clippy::no_effect)] // Fixes a false positive in Rust < 1.78
        *INITIALIZE_CSS;

        let current_child = settings.extra_child.clone();

        let model = Alert {
            settings,
            is_active: false,
            current_child,
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
        // Update the view to contain the extra component, by removing whatever's present in the UI and then adding what the caller's current widget is.
        if let Some(widget) = self.current_child.take() {
            widgets.message_area.remove(&widget);
        }

        if let Some(extra_child) = self.settings.extra_child.clone() {
            widgets.message_area.append(&extra_child);
            self.current_child = Some(extra_child);
        }

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
