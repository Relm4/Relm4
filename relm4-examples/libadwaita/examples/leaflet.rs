use relm4::{adw, gtk, send, AppUpdate, Model, RelmApp, Sender, Widgets};

use adw::prelude::*;
use adw::{ApplicationWindow, HeaderBar, Leaflet};
use gtk::{Align, Box, Button, Label, Orientation, Stack, StackSidebar};

pub struct AppModel {
    message: Option<AppMsg>,
}

pub enum AppMsg {
    UnFolded,
    Folded,
    GoNext,
    GoBack
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        self.message = Some(msg);
        true
    }
}

#[relm4::widget(pub)]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = ApplicationWindow {
            set_default_size: args!(800, 450),
            set_content: leaflet = Some(&Leaflet) {
                append: sidebar = &Box {
                    set_width_request: 280,
                    set_orientation: Orientation::Vertical,
                    append: sidebar_header = &HeaderBar {
                        pack_end: go_next_button = &Button {
                            set_icon_name: "go-next-symbolic",
                            set_visible: false,
                            connect_clicked(sender) => move |_| {
                                send!(sender, AppMsg::GoNext);
                            }
                        },
                        set_show_start_title_buttons: false,
                        set_show_end_title_buttons: false,
                        set_title_widget: Some(&Label::new(Some("Sidebar")))
                    },
                    append = &StackSidebar {
                        set_vexpand: true,
                        set_stack: &stack
                    }
                },
                append: &gtk::Separator::new(Orientation::Horizontal),
                append: content = &Box {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_orientation: Orientation::Vertical,
                    append = &HeaderBar {
                        pack_start: go_back_button = &Button {
                            set_icon_name: "go-previous-symbolic",
                            set_visible: false,
                            connect_clicked(sender) => move |_| {
                                send!(sender, AppMsg::GoBack);
                            }
                        },
                        set_title_widget = Some(&Label) {
                            set_label: "Contents"
                        },
                    },
                    append = &Box {
                        set_vexpand: true,
                        set_valign: Align::Center,
                        set_halign: Align::Center,
                        append: stack = &Stack {
                            set_vexpand: true,
                            add_titled(None, "Step 1") = &Box {
                                set_halign: Align::Center,
                                append: &Label::new(Some("Try switch from the sidebar."))
                            },
                            add_titled(None, "Step 2") = &Box {
                                set_halign: Align::Center,
                                append: &Label::new(Some("Try reducing the window width."))
                            },
                            add_titled(None, "Step 3") = &Box {
                                set_halign: Align::Center,
                                append: &Label::new(Some("Try clicking the go back/forward button above on compact mode."))
                            },
                        }
                    }
                },
                connect_folded_notify(sender) => move |leaflet| {
                    if leaflet.is_folded() {
                        send!(sender, AppMsg::Folded);
                    } else {
                        send!(sender, AppMsg::UnFolded);
                    }
                },
            }
        }
    }

    fn pre_view() {
        if let Some(msg) = &model.message {
            match msg {
                AppMsg::Folded => {
                    self.leaflet.set_visible_child(&self.content);
                    self.go_back_button.set_visible(true);
                    self.go_next_button.set_visible(true);
                    sidebar_header.set_show_start_title_buttons(true);
                    sidebar_header.set_show_end_title_buttons(true);
                }
                AppMsg::UnFolded => {
                    self.go_back_button.set_visible(false);
                    self.go_next_button.set_visible(false);
                    sidebar_header.set_show_start_title_buttons(false);
                    sidebar_header.set_show_end_title_buttons(false);
                },
                AppMsg::GoNext => self.leaflet.set_visible_child(&self.content),
                AppMsg::GoBack => self.leaflet.set_visible_child(&self.sidebar)
            }
        }
    }
}

fn main() {
    let model = AppModel { message: None };
    let app = RelmApp::new(model);
    app.run()
}
