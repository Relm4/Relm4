use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
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

#[derive(Debug)]
enum Input {
    OpenRequest,
    OpenResponse(PathBuf),
    SaveRequest,
    SaveResponse(PathBuf),
    ShowMessage(String),
    ResetMessage,
    Ignore,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Input;
    type Output = ();

    view! {
        root = gtk::ApplicationWindow {
            set_default_size: (600, 400),

            #[watch]
            set_title: Some(model.file_name.as_deref().unwrap_or_default()),

            #[wrap(Some)]
            set_titlebar = &gtk::HeaderBar {
                pack_start = &gtk::Button {
                    set_label: "Open",
                    connect_clicked => Input::OpenRequest,
                },
                pack_end = &gtk::Button {
                    set_label: "Save As",
                    connect_clicked => Input::SaveRequest,

                    #[watch]
                    set_sensitive: model.file_name.is_some(),
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,

                gtk::ScrolledWindow {
                    set_min_content_height: 380,

                    #[wrap(Some)]
                    set_child = &gtk::TextView {
                        set_buffer: Some(&model.buffer),

                        #[watch]
                        set_visible: model.file_name.is_some(),
                    },
                },
            }
        }
    }

    fn post_view() {
        if let Some(text) = &model.message {
            let dialog = gtk::MessageDialog::builder()
                .text(text)
                .use_markup(true)
                .transient_for(&widgets.root)
                .modal(true)
                .buttons(gtk::ButtonsType::Ok)
                .build();
            dialog.connect_response(|dialog, _| dialog.destroy());
            dialog.set_visible(true);
            sender.input(Input::ResetMessage);
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => Input::OpenResponse(path),
                OpenDialogResponse::Cancel => Input::Ignore,
            });

        let save_dialog = SaveDialog::builder()
            .transient_for_native(&root)
            .launch(SaveDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                SaveDialogResponse::Accept(path) => Input::SaveResponse(path),
                SaveDialogResponse::Cancel => Input::Ignore,
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
            "A simple text editor showing the usage of\n<b>OpenFileDialog</b> and <b>SaveFileDialog</b> components.\n\nStart by clicking <b>Open</b> on the header bar.",
        )));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
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
                        "File saved successfully at {path:?}"
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
            Input::Ignore => {}
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.file_dialogs");
    app.run::<App>(());
}
