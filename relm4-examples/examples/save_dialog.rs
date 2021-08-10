use gtk::prelude::{BoxExt, ButtonExt, Cast, GtkWindowExt, OrientableExt};
use relm4::{
    send, AppUpdate, Components, Model, RelmApp, RelmComponent, Sender, WidgetPlus, Widgets,
};
use relm4_components::save_dialog::{
    SaveDialogModel, SaveDialogMsg, SaveDialogParent, SaveDialogParentWidgets, SaveDialogSettings,
};

use std::path::PathBuf;

#[derive(Default)]
struct AppModel {
    counter: u8,
}

enum AppMsg {
    Increment,
    Decrement,
    SaveRequest,
    SaveResponse(PathBuf),
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
            AppMsg::SaveRequest => {
                components
                    .dialog
                    .send(SaveDialogMsg::SaveAs(format!("Counter_{}", self.counter)))
                    .unwrap();
            }
            AppMsg::SaveResponse(path) => {
                println!("File would have been saved at {:?}", path);
            }
        }
        true
    }
}

impl SaveDialogParent for AppModel {
    fn dialog_config(&self) -> SaveDialogSettings {
        SaveDialogSettings {
            accept_label: "Open".to_string(),
            cancel_label: "Cancel".to_string(),
            create_folders: true,
            is_modal: true,
            filters: Vec::new(),
        }
    }

    fn save_msg(path: PathBuf) -> Self::Msg {
        AppMsg::SaveResponse(path)
    }
}

impl SaveDialogParentWidgets for AppWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        Some(self.main_window.clone().upcast::<gtk::Window>())
    }
}

pub struct AppComponents {
    dialog: RelmComponent<SaveDialogModel, AppModel>,
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
                    set_label: "Save",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::SaveRequest);
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
