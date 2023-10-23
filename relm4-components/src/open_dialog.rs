//! Reusable and easily configurable open dialog component.
//!
//! **[Example implementation](https://github.com/Relm4/Relm4/blob/main/relm4-components/examples/file_dialogs.rs)**
use gtk::prelude::{Cast, FileChooserExt, FileExt, ListModelExt, NativeDialogExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use std::{fmt::Debug, marker::PhantomData, path::PathBuf};

/// A component that prompts the user to choose a file.
///
/// The user would be able to select a single file. If you'd like to select multiple, use [`OpenDialogMulti`].
pub type OpenDialog = OpenDialogInner<SingleSelection>;

/// A component that prompts the user to choose a file.
///
/// The user would be able to select multiple files. If you'd like to select just one, use [`OpenDialog`].
pub type OpenDialogMulti = OpenDialogInner<MultiSelection>;

/// Type of selection used for the open dialog.
pub trait Select: Debug {
    /// Output of the selection.
    type Selection: Debug;
    /// Whether to select multiple files inside the dialog.
    const SELECT_MULTIPLE: bool;
    /// Construct selection from the file chooser.
    fn select(dialog: &gtk::FileChooserNative) -> Self::Selection;
}

/// A type of selection where only one file can be chosen at a time.
#[derive(Debug)]
pub struct SingleSelection;

impl Select for SingleSelection {
    type Selection = PathBuf;
    const SELECT_MULTIPLE: bool = false;
    fn select(dialog: &gtk::FileChooserNative) -> Self::Selection {
        dialog
            .file()
            .expect("No file selected")
            .path()
            .expect("No path")
    }
}

/// A type of selection where multiple types can be chosen at a time.
#[derive(Debug)]
pub struct MultiSelection;
impl Select for MultiSelection {
    type Selection = Vec<PathBuf>;
    const SELECT_MULTIPLE: bool = true;
    fn select(dialog: &gtk::FileChooserNative) -> Self::Selection {
        let list_model = dialog.files();
        (0..list_model.n_items())
            .filter_map(|index| list_model.item(index))
            .filter_map(|obj| obj.downcast::<gtk::gio::File>().ok())
            .filter_map(|file| file.path())
            .collect()
    }
}

#[derive(Clone, Debug)]
/// Configuration for the open dialog component
pub struct OpenDialogSettings {
    /// Select folders instead of files
    pub folder_mode: bool,
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
            accept_label: String::from("Open"),
            cancel_label: String::from("Cancel"),
            create_folders: true,
            is_modal: true,
            filters: Vec::new(),
        }
    }
}

#[derive(Debug)]
/// Model for the open dialog component
pub struct OpenDialogInner<S: Select> {
    visible: bool,
    _phantom: PhantomData<S>,
}

/// Messages that can be sent to the open dialog component
#[derive(Debug, Clone)]
pub enum OpenDialogMsg {
    /// Show the dialog
    Open,
    #[doc(hidden)]
    Hide,
}

/// Messages that can be sent from the open dialog component
#[derive(Debug, Clone)]
pub enum OpenDialogResponse<S: Select> {
    /// User clicked accept button.
    Accept(S::Selection),
    /// User clicked cancel button.
    Cancel,
}

/// Widgets of the open dialog component.
#[relm4::component(pub)]
impl<S: Select + 'static> SimpleComponent for OpenDialogInner<S> {
    type Init = OpenDialogSettings;
    type Input = OpenDialogMsg;
    type Output = OpenDialogResponse<S>;

    view! {
        gtk::FileChooserNative {
            set_action: if settings.folder_mode {
                gtk::FileChooserAction::SelectFolder
            } else {
                gtk::FileChooserAction::Open
            },

            set_select_multiple: S::SELECT_MULTIPLE,
            set_create_folders: settings.create_folders,
            set_modal: settings.is_modal,
            set_accept_label: Some(&settings.accept_label),
            set_cancel_label: Some(&settings.cancel_label),
            #[iterate]
            add_filter: &settings.filters,

            #[watch]
            set_visible: model.visible,

            connect_response[sender] => move |dialog, res_ty| {
                match res_ty {
                    gtk::ResponseType::Accept => {
                        let selection = S::select(dialog);
                        sender.output(OpenDialogResponse::Accept(selection)).unwrap();
                    }
                    _ => sender.output(OpenDialogResponse::Cancel).unwrap(),
                }

                sender.input(OpenDialogMsg::Hide);
            }
        }
    }

    fn init(
        settings: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = OpenDialogInner {
            visible: false,
            _phantom: PhantomData,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
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
