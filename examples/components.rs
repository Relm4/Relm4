// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use std::convert::identity;

use gtk::{glib, prelude::*};
use relm4::prelude::*;

struct Header;

#[relm4::component]
impl SimpleComponent for Header {
    type Init = ();
    type Input = ();
    type Output = AppMsg;

    view! {
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",
                append: group = &gtk::ToggleButton {
                    set_label: "View",
                    set_active: true,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::View)).unwrap();
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Edit",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Edit)).unwrap();
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Export",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Export)).unwrap();
                        }
                    },
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Header;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

struct Dialog {
    hidden: bool,
}

#[derive(Debug)]
enum DialogMsg {
    Show,
    Accept,
    Cancel,
}

struct DialogInit {
    text: String,
    secondary_text: Option<String>,
    accept_text: String,
    cancel_text: String,
}

#[relm4::component]
impl SimpleComponent for Dialog {
    type Init = DialogInit;
    type Input = DialogMsg;
    type Output = AppMsg;

    view! {
        dialog = gtk::MessageDialog {
            set_modal: true,
            set_text: Some(&init.text),
            set_secondary_text: init.secondary_text.as_deref(),
            add_button: (&init.accept_text, gtk::ResponseType::Accept),
            add_button: (&init.cancel_text, gtk::ResponseType::Cancel),

            #[watch]
            set_visible: !model.hidden,

            connect_response[sender] => move |_, resp| {
                sender.input(if resp == gtk::ResponseType::Accept {
                    DialogMsg::Accept
                } else {
                    DialogMsg::Cancel
                });
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dialog { hidden: true };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            DialogMsg::Show => self.hidden = false,
            DialogMsg::Accept => {
                self.hidden = true;
                sender.output(AppMsg::Close).unwrap();
            }
            DialogMsg::Cancel => self.hidden = true,
        }
    }
}

#[derive(Debug)]
enum AppMode {
    View,
    Edit,
    Export,
}

#[derive(Debug)]
enum AppMsg {
    SetMode(AppMode),
    CloseRequest,
    Close,
}

struct App {
    mode: AppMode,
    dialog: Controller<Dialog>,
    header: Controller<Header>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_default_size: (500, 250),
            set_titlebar: Some(model.header.widget()),

            #[wrap(Some)]
            set_child = &gtk::Label {
                #[watch]
                set_label: &format!("Placeholder for {:?}", model.mode),
            },
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::CloseRequest);
                glib::Propagation::Proceed
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Header::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let dialog = Dialog::builder()
            .transient_for(&root)
            .launch(DialogInit {
                text: "Do you want to close before saving?".to_string(),
                secondary_text: Some("All unsaved changes will be lost".to_string()),
                accept_text: "Close".to_string(),
                cancel_text: "Cancel".to_string(),
            })
            .forward(sender.input_sender(), identity);

        let model = App {
            mode: AppMode::View,
            header,
            dialog,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SetMode(mode) => {
                self.mode = mode;
            }
            AppMsg::CloseRequest => {
                self.dialog.emit(DialogMsg::Show);
            }
            AppMsg::Close => {
                relm4::main_application().quit();
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.components");
    app.run::<App>(());
}
