// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use std::convert::identity;

use gtk::prelude::{BoxExt, ButtonExt, DialogExt, GtkWindowExt, ToggleButtonExt, WidgetExt};
use relm4::gtk::prelude::Cast;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    MessageBroker, RelmApp, SimpleComponent,
};

static HEADER_BROKER: MessageBroker<bool> = MessageBroker::new();

struct Header(bool);

#[relm4::component]
impl SimpleComponent for Header {
    type Init = ();
    type Input = bool;
    type Output = AppMsg;

    view! {
        gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                add_css_class: "linked",
                append: group = &gtk::ToggleButton {
                    set_label: "View",
                    set_active: true,
                    #[watch]
                    set_has_frame: model.0,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::View)).unwrap();
                        }
                    },
                },
                gtk::ToggleButton {
                    set_label: "Edit",
                    set_group: Some(&group),
                    #[watch]
                    set_has_frame: model.0,
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Edit)).unwrap();
                        }
                    },
                },
                gtk::ToggleButton {
                    set_group: Some(&group),
                    set_label: "Export",
                    #[watch]
                    set_has_frame: model.0,
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
        let model = Header(false);
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        self.0 = input;
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

#[relm4::component]
impl SimpleComponent for Dialog {
    type Init = gtk::Window;
    type Input = DialogMsg;
    type Output = AppMsg;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            set_text: Some("Do you want frames around the header buttons?"),
            add_button: ("Yes", gtk::ResponseType::Accept),
            add_button: ("No", gtk::ResponseType::Cancel),

            #[watch]
            set_visible: !model.hidden,

            connect_response[sender] => move |_, resp| {
                sender.input(if resp == gtk::ResponseType::Accept {
                    // Send a message directly to another component that's
                    // not the parent component!
                    HEADER_BROKER.send(true);
                    DialogMsg::Accept
                } else {
                    HEADER_BROKER.send(false);
                    DialogMsg::Cancel
                });
            }
        }
    }

    fn init(
        parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dialog { hidden: true };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            DialogMsg::Show => self.hidden = false,
            DialogMsg::Cancel | DialogMsg::Accept => self.hidden = true,
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
    ShowDialog,
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
            gtk::Button {
                set_label: "Change header style",
                connect_clicked => AppMsg::ShowDialog,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Header::builder()
            .launch_with_broker((), &HEADER_BROKER)
            .forward(sender.input_sender(), identity);

        let dialog = Dialog::builder()
            .launch(root.clone().upcast())
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
            AppMsg::ShowDialog => {
                self.dialog.emit(DialogMsg::Show);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.message_broker");
    app.run::<App>(());
}
