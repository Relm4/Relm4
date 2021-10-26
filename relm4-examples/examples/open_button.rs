use gtk::prelude::{Cast, GtkWindowExt};
use relm4::{AppUpdate, Components, Model, RelmApp, RelmComponent, Sender, Widgets};
use relm4_components::open_button::{
    OpenButtonConfig, OpenButtonModel, OpenButtonParent, OpenButtonSettings,
};
use relm4_components::open_dialog::{OpenDialogConfig, OpenDialogSettings};
use relm4_components::ParentWindow;

use std::path::PathBuf;

struct AppModel {}

enum AppMsg {
    Open(PathBuf),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        msg: AppMsg,
        _components: &AppComponents,
        _sender: Sender<AppMsg>,
    ) -> bool {
        match msg {
            AppMsg::Open(path) => {
                println!("* Open file at {:?} *", path);
            }
        }

        true
    }
}

struct OpenFileButtonConfig {}

impl OpenDialogConfig for OpenFileButtonConfig {
    type Model = AppModel;

    fn open_dialog_config(_model: &Self::Model) -> OpenDialogSettings {
        OpenDialogSettings {
            accept_label: "Open",
            cancel_label: "Cancel",
            create_folders: true,
            is_modal: true,
            filters: Vec::new(),
        }
    }
}

impl OpenButtonConfig for OpenFileButtonConfig {
    fn open_button_config(_model: &Self::Model) -> OpenButtonSettings {
        OpenButtonSettings {
            text: "Open file",
            recently_opened_files: Some(".recent_files"),
            max_recent_files: 10,
        }
    }
}

impl OpenButtonParent for AppModel {
    fn open_msg(path: PathBuf) -> Self::Msg {
        AppMsg::Open(path)
    }
}

impl ParentWindow for AppWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        Some(self.main_window.clone().upcast::<gtk::Window>())
    }
}

pub struct AppComponents {
    open_button: RelmComponent<OpenButtonModel<OpenFileButtonConfig>, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        model: &AppModel,
        parent_widgets: &AppWidgets,
        sender: Sender<AppMsg>,
    ) -> Self {
        AppComponents {
            open_button: RelmComponent::new(model, parent_widgets, sender),
        }
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_default_width: 300,
            set_default_height: 100,
            set_titlebar = Some(&gtk::HeaderBar) {
                pack_start: component!(open_button),
            }
        }
    }
}

fn main() {
    let model = AppModel {};
    let app = RelmApp::new(model);
    app.run();
}
