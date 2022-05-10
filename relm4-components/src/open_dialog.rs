//! Reusable and easily configurable open dialog component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/next/examples/file_dialogs.rs)**
use gtk::prelude::{Cast, FileChooserExt, FileExt, ListModelExt, NativeDialogExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use std::path::PathBuf;

#[derive(Clone, Debug)]
/// Configuration for the open dialog component
pub struct OpenDialogSettings {
    /// Select folders instead of files.
    ///
    /// You should be aware the user might be able to select folders
    /// even if this setting is set to `false`. This is a technical limitation of gtk.
    pub folder_mode: bool,
    /// Allow selection of multiple items.
    pub select_multiple: bool,
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

impl Default for OpenDialogSettings {
    fn default() -> Self {
        OpenDialogSettings {
            folder_mode: false,
            select_multiple: false,
            accept_label: String::from("Open"),
            cancel_label: String::from("Cancel"),
            create_folders: true,
            is_modal: true,
            filters: Vec::new(),
        }
    }
}

#[derive(Debug)]
/// A model for the open dialog component
pub struct OpenDialog {
    visible: bool,
}

/// Messages that can be sent to the open dialog component
#[derive(Debug, Clone)]
pub enum OpenDialogMsg {
    /// Show the dialog
    Open,
    #[doc(hidden)]
    Hide,
}

/// Messages that can be sent to the open dialog component
#[derive(Debug, Clone)]
pub enum OpenDialogResponse {
    /// User clicked accept button. Requires `select_multiple` to be `true`.
    AcceptMultiple(Vec<PathBuf>),
    /// User clicked accept button. Requires `select_multiple` to be `false` (default behavior).
    Accept(PathBuf),
    /// User clicked cancel button.
    Cancel,
}

/// Widgets of the open dialog component.
#[relm4::component(pub)]
impl SimpleComponent for OpenDialog {
    type Widgets = OpenDialogWidgets;

    type InitParams = OpenDialogSettings;

    type Input = OpenDialogMsg;
    type Output = OpenDialogResponse;

    view! {
        gtk::FileChooserNative {
            set_action: if settings.folder_mode {
                gtk::FileChooserAction::SelectFolder
            } else {
                gtk::FileChooserAction::Open
            },

            set_select_multiple: settings.select_multiple,
            set_create_folders: settings.create_folders,
            set_modal: settings.is_modal,
            set_accept_label: Some(&settings.accept_label),
            set_cancel_label: Some(&settings.cancel_label),
            add_filter: iterate!(&settings.filters),

            set_visible: watch!(model.visible),

            connect_response(sender) => move |dialog, res_ty| {
                match res_ty {
                    gtk::ResponseType::Accept => {
                        match settings.select_multiple {
                            true => {
                                let list_model = dialog.files();
                                let paths = (0..list_model.n_items())
                                    .filter_map(|index| list_model.item(index))
                                    .filter_map(|obj| obj.downcast::<gtk::gio::File>().ok())
                                    .filter_map(|file| file.path()).collect();
                                sender.output(OpenDialogResponse::AcceptMultiple(paths));
                                sender.input(OpenDialogMsg::Hide);
                                return;
                            },
                            false => {
                                if let Some(file) = dialog.file() {
                                    if let Some(path) = file.path() {
                                        sender.output(OpenDialogResponse::Accept(path));
                                        sender.input(OpenDialogMsg::Hide);
                                        return;
                                    }
                                }
                               sender.output(OpenDialogResponse::Cancel);
                            }
                        }
                    }
                    _ => sender.output(OpenDialogResponse::Cancel),
                }

                sender.input(OpenDialogMsg::Hide);
            }
        }
    }

    fn init(
        settings: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = OpenDialog { visible: false };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: &ComponentSender<Self>) {
        match message {
            OpenDialogMsg::Open => {
                self.visible = true;
            }
            OpenDialogMsg::Hide => {
                self.visible = false;
            }
        }
    }
}
