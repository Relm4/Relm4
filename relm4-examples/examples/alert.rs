use gtk::prelude::{BoxExt, ButtonExt, Cast, GtkWindowExt, OrientableExt};
use relm4::{gtk, send, AppUpdate, Model, RelmApp, RelmComponent, Sender, WidgetPlus, Widgets};
use relm4_components::alert::{AlertConfig, AlertModel, AlertMsg, AlertParent, AlertSettings};
use relm4_components::ParentWindow;

#[derive(Default)]
struct AppModel {
    counter: u8,
    alert_toggle: bool,
}

enum AppMsg {
    Increment,
    Decrement,
    CloseRequest,
    Save,
    Close,
    Ignore,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
            AppMsg::CloseRequest => {
                if self.counter == 42 {
                    return false;
                } else {
                    self.alert_toggle = !self.alert_toggle;
                    if self.alert_toggle {
                        components.dialog.send(AlertMsg::Show).unwrap();
                    } else {
                        components.second_dialog.send(AlertMsg::Show).unwrap();
                    }
                }
            }
            AppMsg::Save => {
                println!("* Open save dialog here *");
            }
            AppMsg::Close => {
                return false;
            }
            AppMsg::Ignore => (),
        }

        true
    }
}

struct FirstAlert {}

impl AlertConfig for FirstAlert {
    type Model = AppModel;

    fn alert_config(_model: &AppModel) -> AlertSettings {
        AlertSettings {
            text: "Do you want to quit without saving? (First alert)",
            secondary_text: Some("Your counter hasn't reached 42 yet"),
            confirm_label: "Close without saving",
            cancel_label: "Cancel",
            option_label: Some("Save"),
            is_modal: true,
            destructive_accept: true,
        }
    }
}

struct SecondAlert {}

impl AlertConfig for SecondAlert {
    type Model = AppModel;

    fn alert_config(_model: &AppModel) -> AlertSettings {
        AlertSettings {
            text: "Do you want to quit without saving? (Second alert)",
            secondary_text: Some("Your counter hasn't reached 42 yet"),
            confirm_label: "Close without saving",
            cancel_label: "Cancel",
            option_label: Some("Save"),
            is_modal: true,
            destructive_accept: true,
        }
    }
}

impl AlertParent for AppModel {
    fn confirm_msg() -> Self::Msg {
        AppMsg::Close
    }

    fn cancel_msg() -> Self::Msg {
        AppMsg::Ignore
    }

    fn option_msg() -> Self::Msg {
        AppMsg::Save
    }
}

impl ParentWindow for AppWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        Some(self.main_window.clone().upcast::<gtk::Window>())
    }
}

#[derive(relm4_macros::Components)]
pub struct AppComponents {
    dialog: RelmComponent<AlertModel<FirstAlert>, AppModel>,
    second_dialog: RelmComponent<AlertModel<SecondAlert>, AppModel>,
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            connect_close_request(sender) => move |_| {
                send!(sender, AppMsg::CloseRequest);
                gtk::Inhibit(true)
            },
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                },
                append = &gtk::Button {
                    set_label: "Decrement",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Decrement);
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                },
                append = &gtk::Button {
                    set_label: "Close",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::CloseRequest);
                    },
                },
            },
        }
    }
}

fn main() {
    let model = AppModel::default();
    let app = RelmApp::new(model);
    app.run();
}
