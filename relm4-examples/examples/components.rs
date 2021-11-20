use gtk::prelude::{BoxExt, ButtonExt, DialogExt, GtkWindowExt, ToggleButtonExt, WidgetExt};
use relm4::{send, AppUpdate, ComponentUpdate, Model, RelmApp, RelmComponent, Sender, Widgets};

enum HeaderMsg {
    View,
    Edit,
    Export,
}

struct HeaderModel {}

impl Model for HeaderModel {
    type Msg = HeaderMsg;
    type Widgets = HeaderWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for HeaderModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        HeaderModel {}
    }

    fn update(
        &mut self,
        msg: HeaderMsg,
        _components: &(),
        _sender: Sender<HeaderMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            HeaderMsg::View => {
                send!(parent_sender, AppMsg::SetMode(AppMode::View));
            }
            HeaderMsg::Edit => {
                send!(parent_sender, AppMsg::SetMode(AppMode::Edit));
            }
            HeaderMsg::Export => {
                send!(parent_sender, AppMsg::SetMode(AppMode::Export));
            }
        }
    }
}

#[relm4_macros::widget]
impl Widgets<HeaderModel, AppModel> for HeaderWidgets {
    view! {
        gtk::HeaderBar {
            set_title_widget = Some(&gtk::Box) {
                add_css_class: "linked",
                append: group = &gtk::ToggleButton {
                    set_label: "View",
                    set_active: true,
                    connect_toggled(sender) => move |btn| {
                        if btn.is_active() {
                            send!(sender, HeaderMsg::View);
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Edit",
                    set_group: Some(&group),
                    connect_toggled(sender) => move |btn| {
                        if btn.is_active() {
                            send!(sender, HeaderMsg::Edit);
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Export",
                    set_group: Some(&group),
                    connect_toggled(sender) => move |btn| {
                        if btn.is_active() {
                            send!(sender, HeaderMsg::Export);
                        }
                    },
                },
            }
        }
    }
}

struct DialogModel {
    hidden: bool,
}

enum DialogMsg {
    Show,
    Accept,
    Cancel,
}

impl Model for DialogModel {
    type Msg = DialogMsg;
    type Widgets = DialogWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for DialogModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        DialogModel { hidden: true }
    }

    fn update(
        &mut self,
        msg: DialogMsg,
        _components: &(),
        _sender: Sender<DialogMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            DialogMsg::Show => self.hidden = false,
            DialogMsg::Accept => {
                self.hidden = true;
                send!(parent_sender, AppMsg::Close);
            }
            DialogMsg::Cancel => self.hidden = true,
        }
    }
}

#[relm4_macros::widget]
impl Widgets<DialogModel, AppModel> for DialogWidgets {
    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_text: Some("Do you want to close before saving?"),
            set_secondary_text: Some("All unsaved changes will be lost"),
            add_button: args!("Close", gtk::ResponseType::Accept),
            add_button: args!("Cancel", gtk::ResponseType::Cancel),
            connect_response(sender) => move |_, resp| {
                send!(sender, if resp == gtk::ResponseType::Accept {
                    DialogMsg::Accept
                } else {
                    DialogMsg::Cancel
                });
            }
        }
    }
}

#[derive(relm4_macros::Components)]
struct AppComponents {
    header: RelmComponent<HeaderModel, AppModel>,
    dialog: RelmComponent<DialogModel, AppModel>,
}

#[derive(Debug)]
enum AppMode {
    View,
    Edit,
    Export,
}

enum AppMsg {
    SetMode(AppMode),
    CloseRequest,
    Close,
}

struct AppModel {
    mode: AppMode,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_default_width: 500,
            set_default_height: 250,
            set_titlebar: Some(components.header.root_widget()),
            set_child = Some(&gtk::Label) {
                set_label: watch!(&format!("Placeholder for {:?}", model.mode)),
            },
            connect_close_request(sender) => move |_| {
                send!(sender, AppMsg::CloseRequest);
                gtk::Inhibit(true)
            }
        }
    }
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::SetMode(mode) => {
                self.mode = mode;
            }
            AppMsg::CloseRequest => {
                components.dialog.send(DialogMsg::Show).unwrap();
            }
            AppMsg::Close => {
                return false;
            }
        }
        true
    }
}

fn main() {
    let model = AppModel {
        mode: AppMode::View,
    };
    let relm = RelmApp::new(model);
    relm.run();
}
