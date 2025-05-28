use gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
    gtk::{self, glib},
};
use relm4_components::alert::{Alert, AlertMsg, AlertResponse, AlertSettings};

struct App {
    counter: u8,
    alert_toggle: bool,
    dialog: Controller<Alert>,
    second_dialog: Controller<Alert>,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
    CloseRequest,
    Save,
    Close,
    Ignore,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Alert example"),
            set_default_size: (300, 100),

            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::CloseRequest);
                glib::Propagation::Proceed
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked => AppMsg::Increment,
                },
                append = &gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => AppMsg::Decrement,
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },
                append = &gtk::Button {
                    set_label: "Close",
                    connect_clicked => AppMsg::CloseRequest,
                },
            },
        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
            AppMsg::CloseRequest => {
                if self.counter == 42 {
                    relm4::main_application().quit();
                } else {
                    self.alert_toggle = !self.alert_toggle;
                    if self.alert_toggle {
                        self.dialog.emit(AlertMsg::Show);
                    } else {
                        self.second_dialog.emit(AlertMsg::Show);
                    }
                }
            }
            AppMsg::Save => {
                println!("* Open save dialog here *");
            }
            AppMsg::Close => {
                relm4::main_application().quit();
            }
            AppMsg::Ignore => (),
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            counter: 0,
            alert_toggle: false,
            dialog: Alert::builder()
                .transient_for(&root)
                .launch(AlertSettings {
                    text: Some(String::from(
                        "Do you want to quit without saving? (First alert)",
                    )),
                    secondary_text: Some(String::from("Your counter hasn't reached 42 yet")),
                    confirm_label: Some(String::from("Close without saving")),
                    cancel_label: Some(String::from("Cancel")),
                    option_label: Some(String::from("Save")),
                    is_modal: true,
                    destructive_accept: true,
                    extra_child: Some(gtk::Button::with_label("Button in Alert").into()),
                })
                .forward(sender.input_sender(), convert_alert_response),
            second_dialog: Alert::builder()
                .transient_for(&root)
                .launch(AlertSettings {
                    text: Some(String::from(
                        "Do you want to quit without saving? (Second alert)",
                    )),
                    secondary_text: Some(String::from("Your counter hasn't reached 42 yet")),
                    confirm_label: Some(String::from("Close without saving")),
                    cancel_label: Some(String::from("Cancel")),
                    option_label: Some(String::from("Save")),
                    is_modal: true,
                    destructive_accept: true,
                    extra_child: None,
                })
                .forward(sender.input_sender(), convert_alert_response),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn convert_alert_response(response: AlertResponse) -> AppMsg {
    match response {
        AlertResponse::Confirm => AppMsg::Close,
        AlertResponse::Cancel => AppMsg::Ignore,
        AlertResponse::Option => AppMsg::Save,
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.alert");
    app.run::<App>(());
}
