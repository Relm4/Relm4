//! Reusable and easily configurable save dialog component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/save_dialog.rs)**

use gtk::prelude::{FileChooserExt, FileExt, NativeDialogExt};
use relm4::{send, ComponentUpdate, Model, Sender};

use std::path::PathBuf;

pub struct SaveDialogSettings {
    /// Label for cancel button
    pub cancel_label: String,
    /// Label for accept button
    pub accept_label: String,
    /// Allow or disallow creating folders
    pub create_folders: bool,
    /// Modal dialogs freeze other windows as long they are visible
    pub is_modal: bool,
    /// Filter for MINE types or other patterns
    pub filters: Vec<gtk::FileFilter>,
}

#[tracker::track]
pub struct SaveDialogModel {
    #[do_not_track]
    settings: SaveDialogSettings,
    suggestion: Option<String>,
    is_active: bool,
    name: String,
}

pub enum SaveDialogMsg {
    /// Opens the dialog
    Save,
    /// Opens the dialog with a suggested file name
    SaveAs(String),
    #[doc(hidden)]
    Accept(PathBuf),
    #[doc(hidden)]
    InvalidInput,
    #[doc(hidden)]
    Cancel,
}

impl Model for SaveDialogModel {
    type Msg = SaveDialogMsg;
    type Widgets = SaveDialogWidgets;
    type Components = ();
}

/// Interface for the parent model
pub trait SaveDialogParent: Model
where
    Self::Widgets: SaveDialogParentWidgets,
{
    /// Configure the save dialog
    fn dialog_config(&self) -> SaveDialogSettings;

    /// Tell the save dialog how to response if the user wants to save
    fn save_msg(path: PathBuf) -> Self::Msg;
}

/// Get the parent window that allows setting the parent window of the dialog with
/// [`gtk::prelude::GtkWindowExt::set_transient_for`].
pub trait SaveDialogParentWidgets {
    fn parent_window(&self) -> Option<gtk::Window>;
}

impl<ParentModel> ComponentUpdate<ParentModel> for SaveDialogModel
where
    ParentModel: SaveDialogParent,
    <ParentModel as relm4::Model>::Widgets: SaveDialogParentWidgets,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        SaveDialogModel {
            settings: parent_model.dialog_config(),
            is_active: false,
            suggestion: None,
            name: String::new(),
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: SaveDialogMsg,
        _components: &(),
        _sender: Sender<SaveDialogMsg>,
        parent_sender: Sender<ParentModel::Msg>,
    ) {
        self.reset();

        match msg {
            SaveDialogMsg::Save => {
                self.is_active = true;
            }
            SaveDialogMsg::SaveAs(name) => {
                self.is_active = true;
                self.set_name(name);
            }
            SaveDialogMsg::Cancel => {
                self.is_active = false;
            }
            SaveDialogMsg::Accept(path) => {
                self.is_active = false;
                parent_sender.send(ParentModel::save_msg(path)).unwrap();
            }
            _ => (),
        }
    }
}

#[relm4_macros::widget(pub)]
impl<ParentModel> relm4::Widgets<SaveDialogModel, ParentModel> for SaveDialogWidgets
where
    ParentModel: Model,
    ParentModel::Widgets: SaveDialogParentWidgets,
{
    //type Model = SaveDialogModel;

    view! {
        gtk::FileChooserNative {
            set_action: gtk::FileChooserAction::Save,
            set_visible: watch!(model.is_active),
            set_current_name: track!(model.changed(SaveDialogModel::name()), &model.name),
            add_filter: iterate!(&model.settings.filters),
            set_create_folders: model.settings.create_folders,
            set_cancel_label: Some(&model.settings.cancel_label),
            set_accept_label: Some(&model.settings.accept_label),
            set_modal: model.settings.is_modal,
            set_transient_for: parent_widgets.parent_window().as_ref(),
            connect_response => move |dialog, res_ty| {
                match res_ty {
                    gtk::ResponseType::Accept => {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                send!(sender, SaveDialogMsg::Accept(path));
                                return;
                            }
                        }
                        send!(sender, SaveDialogMsg::InvalidInput);
                    },
                    gtk::ResponseType::Cancel => {
                        send!(sender, SaveDialogMsg::Cancel)
                    },
                    _ => (),
                }
            },
        }
    }
}
