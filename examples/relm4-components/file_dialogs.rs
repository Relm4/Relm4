use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent, WidgetPlus,
};
use relm4_components::{open_dialog::*, save_dialog::*};

use std::path::PathBuf;

struct App {
    open_dialog: Controller<OpenDialog>,
    save_dialog: Controller<SaveDialog>,
    buffer: gtk::TextBuffer,
    file_name: Option<String>,
    message: Option<String>,
}

enum Input {
    OpenRequest,
    OpenResponse(PathBuf),
    SaveRequest,
    SaveResponse(PathBuf),
    ShowMessage(String),
    ResetMessage,
}

#[relm4::component]
impl SimpleComponent for App {
    type Widgets = AppWidgets;

    type InitParams = ();

    type Input = Input;
    type Output = ();

    view! {
        root = gtk::ApplicationWindow {
            set_title: watch!(Some(model.file_name.as_deref().unwrap_or_default())),
            set_default_width: 600,
            set_default_height: 400,

            set_titlebar = Some(&gtk::HeaderBar) {
                pack_start = &gtk::Button {
                    set_label: "Open",
                    connect_clicked(sender) => move |_| {
                        sender.input(Input::OpenRequest);
                    },
                },
                pack_end = &gtk::Button {
                    set_label: "Save",
                    set_sensitive: watch!(model.file_name.is_some()),
                    connect_clicked(sender) => move |_| {
                        sender.input(Input::SaveRequest);
                    },
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,

                gtk::ScrolledWindow {
                    set_min_content_height: 380,

                    set_child = Some(&gtk::TextView) {
                        set_visible: watch!(model.file_name.is_some()),
                        set_buffer: Some(&model.buffer),
                    },
                },
            }
        }
    }

    fn post_view() {
        if let Some(text) = &model.message {
            let dialog = gtk::MessageDialog::builder()
                .text(text)
                .transient_for(&widgets.root)
                .modal(true)
                .buttons(gtk::ButtonsType::Ok)
                .build();
            dialog.connect_response(|dialog, _| dialog.destroy());
            dialog.show();
            sender.input(Input::ResetMessage);
        }
    }

    fn init(
        _: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => Input::OpenResponse(path),
                OpenDialogResponse::Cancel => {
                    Input::ShowMessage(String::from("File opening was cancelled"))
                }
            });

        let save_dialog = SaveDialog::builder()
            .transient_for_native(root)
            .launch(SaveDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                SaveDialogResponse::Accept(path) => Input::SaveResponse(path),
                SaveDialogResponse::Cancel => {
                    Input::ShowMessage(String::from("File saving was cancelled"))
                }
            });

        let model = App {
            open_dialog,
            save_dialog,
            buffer: gtk::TextBuffer::new(None),
            file_name: None,
            message: None,
        };

        let widgets = view_output!();

        sender.input(Input::ShowMessage(String::from(
            "This is a simple text editor. Start by clicking \"Open\" on the header bar.",
        )));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {
        match message {
            Input::OpenRequest => self.open_dialog.emit(OpenDialogMsg::Open),
            Input::OpenResponse(path) => match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    self.buffer.set_text(&contents);
                    self.file_name = Some(
                        path.file_name()
                            .expect("The path has no file name")
                            .to_str()
                            .expect("Cannot convert file name to string")
                            .to_string(),
                    );
                }
                Err(e) => sender.input(Input::ShowMessage(e.to_string())),
            },
            Input::SaveRequest => self
                .save_dialog
                .emit(SaveDialogMsg::SaveAs(self.file_name.clone().unwrap())),
            Input::SaveResponse(path) => match std::fs::write(
                &path,
                self.buffer
                    .text(&self.buffer.start_iter(), &self.buffer.end_iter(), false),
            ) {
                Ok(_) => {
                    sender.input(Input::ShowMessage(format!(
                        "File saved successfully at {:?}",
                        path
                    )));
                    self.buffer.set_text("");
                    self.file_name = None;
                }
                Err(e) => sender.input(Input::ShowMessage(e.to_string())),
            },
            Input::ShowMessage(message) => {
                self.message = Some(message);
            }
            Input::ResetMessage => {
                self.message = None;
            }
        }
    }
}

fn main() {
    let app: RelmApp<App> = RelmApp::new("relm4.example.file_dialogs");
    app.run(());
}
