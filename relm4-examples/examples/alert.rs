use gtk::prelude::{BoxExt, ButtonExt, Cast, GtkWindowExt, OrientableExt};
use relm4::{
    send, AppUpdate, Components, Model, RelmApp, RelmComponent, Sender, WidgetPlus, Widgets,
};
use relm4_components::alert::{
    AlertModel, AlertMsg, AlertParent, AlertParentWidgets, AlertSettings,
};

#[derive(Default)]
struct AppModel {
    counter: u8,
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
                    components.dialog.send(AlertMsg::Show).unwrap();
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

impl AlertParent for AppModel {
    fn alert_config(&self) -> AlertSettings {
        AlertSettings {
            text: "Do you want to quit without saving?".to_string(),
            secondary_text: Some("Your counter hasn't reached 42 yet".to_string()),
            confirm_label: "Close without saving".to_string(),
            cancel_label: "Cancel".to_string(),
            option_label: Some("Save".to_string()),
            is_modal: true,
            destructive_accept: true,
        }
    }

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

impl AlertParentWidgets for AppWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        Some(self.main_window.clone().upcast::<gtk::Window>())
    }
}

pub struct AppComponents {
    dialog: RelmComponent<AlertModel, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        model: &AppModel,
        parent_widgets: &AppWidgets,
        sender: Sender<AppMsg>,
    ) -> Self {
        AppComponents {
            dialog: RelmComponent::new(model, parent_widgets, sender),
        }
    }
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
