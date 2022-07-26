use std::convert::identity;

use gtk::prelude::{BoxExt, ButtonExt, DialogExt, GtkWindowExt, ToggleButtonExt, WidgetExt};
use relm4::gtk::prelude::{ApplicationExt, Cast};
use relm4::gtk::{self};
use relm4::{
    adw, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent,
};

struct Header;

#[relm4::component]
impl SimpleComponent for Header {
    type InitParams = ();
    type Input = ();
    type Output = AppMsg;
    type Widgets = HeaderWidgets;

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
                            sender.output(AppMsg::SetMode(AppMode::View));
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Edit",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Edit));
                        }
                    },
                },
                append = &gtk::ToggleButton {
                    set_label: "Export",
                    set_group: Some(&group),
                    connect_toggled[sender] => move |btn| {
                        if btn.is_active() {
                            sender.output(AppMsg::SetMode(AppMode::Export));
                        }
                    },
                },
            }
        }
    }

    fn init(
        _init_params: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
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

#[relm4::component]
impl SimpleComponent for Dialog {
    type InitParams = gtk::Window;
    type Input = DialogMsg;
    type Output = AppMsg;
    type Widgets = DialogWidgets;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            set_text: Some("Do you want to close before saving?"),
            set_secondary_text: Some("All unsaved changes will be lost"),
            add_button: ("Close", gtk::ResponseType::Accept),
            add_button: ("Cancel", gtk::ResponseType::Cancel),

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
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dialog { hidden: true };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        match msg {
            DialogMsg::Show => self.hidden = false,
            DialogMsg::Accept => {
                self.hidden = true;
                sender.output(AppMsg::Close);
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
    application: adw::Application,
    dialog: Controller<Dialog>,
    header: Controller<Header>,
}

#[relm4::component]
impl SimpleComponent for App {
    type InitParams = adw::Application;
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        main_window = gtk::ApplicationWindow {
            set_default_width: 500,
            set_default_height: 250,
            set_titlebar: Some(model.header.widget()),

            #[wrap(Some)]
            set_child = &gtk::Label {
                #[watch]
                set_label: &format!("Placeholder for {:?}", model.mode),
            },
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::CloseRequest);
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        application: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header = Header::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let dialog = Dialog::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);

        let model = App {
            mode: AppMode::View,
            header,
            dialog,
            application,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: &ComponentSender<Self>) {
        match msg {
            AppMsg::SetMode(mode) => {
                self.mode = mode;
            }
            AppMsg::CloseRequest => {
                self.dialog.emit(DialogMsg::Show);
            }
            AppMsg::Close => {
                self.application.quit();
            }
        }
    }
}

fn main() {
    let relm_app = RelmApp::new("relm4.test.components");
    let application = relm_app.app.clone();
    relm_app.run::<App>(application);
}
