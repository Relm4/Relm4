//! Reusable and easily configurable save dialog component.
//!
//! **[Example implementation](https://github.com/Relm4/Relm4/blob/main/relm4-components/examples/file_dialogs.rs)**
use gtk::prelude::{FileChooserExt, FileExt, NativeDialogExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use std::path::PathBuf;

#[derive(Clone, Debug)]
/// Configuration for the save dialog component
pub struct SaveDialogSettings {
    /// Label for cancel button
    pub cancel_label: String,
    /// Label for accept button
    pub accept_label: String,
    /// Allow or disallow creating folders
    pub create_folders: bool,
    /// Freeze other windows while the dialog is open
    pub is_modal: bool,
    /// Filter for MIME types or other patterns
    pub filters: Vec<gtk::FileFilter>,
}

impl Default for SaveDialogSettings {
    fn default() -> Self {
        SaveDialogSettings {
            accept_label: String::from("Save"),
            cancel_label: String::from("Cancel"),
            create_folders: true,
            is_modal: true,
            filters: Vec::new(),
        }
    }
}

#[derive(Debug)]
/// A model for the save dialog component
pub struct SaveDialog {
    current_name: String,
    visible: bool,
}

/// Messages that can be sent to the save dialog component
#[derive(Debug, Clone)]
pub enum SaveDialogMsg {
    /// Show the dialog
    Save,
    /// Show the dialog, with a suggested file name
    SaveAs(String),
    #[doc(hidden)]
    Hide,
}

/// Messages that can be sent from the save dialog component
#[derive(Debug, Clone)]
pub enum SaveDialogResponse {
    /// User clicked accept button.
    Accept(PathBuf),
    /// User clicked cancel button.
    Cancel,
}

/// Widgets of the save dialog component.
#[relm4::component(pub)]
impl SimpleComponent for SaveDialog {
    type Init = SaveDialogSettings;

    type Input = SaveDialogMsg;
    type Output = SaveDialogResponse;

    view! {
        gtk::FileChooserNative {
            set_action: gtk::FileChooserAction::Save,

            set_create_folders: settings.create_folders,
            set_modal: settings.is_modal,
            set_accept_label: Some(&settings.accept_label),
            set_cancel_label: Some(&settings.cancel_label),
            #[iterate]
            add_filter: &settings.filters,

            #[watch]
            set_current_name: &model.current_name,
            #[watch]
            set_visible: model.visible,

            connect_response[sender] => move |dialog, res_ty| {
                match res_ty {
                    gtk::ResponseType::Accept => {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                sender.output(SaveDialogResponse::Accept(path)).unwrap();
                                sender.input(SaveDialogMsg::Hide);
                                return;
                            }
                        }
                        sender.output(SaveDialogResponse::Cancel).unwrap();
                    }
                    _ => sender.output(SaveDialogResponse::Cancel).unwrap(),
                }
                sender.input(SaveDialogMsg::Hide);
            }
        }
    }

    fn init(
        settings: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SaveDialog {
            current_name: String::new(),
            visible: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SaveDialogMsg::Save => {
                self.current_name = String::new();
                self.visible = true;
            }
            SaveDialogMsg::SaveAs(file_name) => {
                self.current_name = file_name;
                self.visible = true;
            }
            SaveDialogMsg::Hide => {
                self.visible = false;
            }
        }
    }
}
