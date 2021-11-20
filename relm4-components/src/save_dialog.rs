//! Reusable and easily configurable save dialog component.
//!
//! **[Example implementation](https://github.com/AaronErhardt/relm4/blob/main/relm4-examples/examples/save_dialog.rs)**

use gtk::prelude::{FileChooserExt, FileExt, NativeDialogExt};
use relm4::{send, ComponentUpdate, Model, Sender};

use std::marker::PhantomData;
use std::path::PathBuf;

use crate::ParentWindow;

#[derive(Clone, Debug)]
/// Configuration for the save dialog component
pub struct SaveDialogSettings {
    /// Label for cancel button
    pub cancel_label: &'static str,
    /// Label for accept button
    pub accept_label: &'static str,
    /// Allow or disallow creating folders
    pub create_folders: bool,
    /// Modal dialogs freeze other windows as long they are visible
    pub is_modal: bool,
    /// Filter for MINE types or other patterns
    pub filters: Vec<gtk::FileFilter>,
}

///Interface for building the configuration for SaveDialog
pub trait SaveDialogConfig {
    /// Model from which configuration should be built
    type Model: Model;
    /// Configure the save dialog
    fn dialog_config(model: &Self::Model) -> SaveDialogSettings;
}

#[tracker::track]
#[derive(Debug)]
/// Model of the save dialog component
pub struct SaveDialogModel<Conf: SaveDialogConfig> {
    #[do_not_track]
    settings: SaveDialogSettings,
    suggestion: Option<String>,
    is_active: bool,
    name: String,
    #[do_not_track]
    _config_provider: PhantomData<*const Conf>, //we don't own Conf, there is no instance of Conf
}

#[derive(Debug)]
/// Messages that can be sent to the save dialog component
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

impl<Conf: SaveDialogConfig> Model for SaveDialogModel<Conf> {
    type Msg = SaveDialogMsg;
    type Widgets = SaveDialogWidgets;
    type Components = ();
}

/// Interface for the parent model of the save dialog
pub trait SaveDialogParent: Model
where
    Self::Widgets: ParentWindow,
{
    /// Tell the save dialog how to response if the user wants to save
    fn save_msg(path: PathBuf) -> Self::Msg;
}

impl<ParentModel, Conf> ComponentUpdate<ParentModel> for SaveDialogModel<Conf>
where
    ParentModel: SaveDialogParent,
    <ParentModel as relm4::Model>::Widgets: ParentWindow,
    Conf: SaveDialogConfig<Model = ParentModel>,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        Self {
            settings: Conf::dialog_config(parent_model),
            is_active: false,
            suggestion: None,
            name: String::new(),
            tracker: 0,
            _config_provider: PhantomData,
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
/// Widgets for the save dialog
impl<ParentModel, Conf> relm4::Widgets<SaveDialogModel<Conf>, ParentModel> for SaveDialogWidgets
where
    ParentModel: Model,
    ParentModel::Widgets: ParentWindow,
    Conf: SaveDialogConfig<Model = ParentModel>,
{
    view! {
        gtk::FileChooserNative {
            set_action: gtk::FileChooserAction::Save,
            set_visible: watch!(model.is_active),
            set_current_name: track!(model.changed(SaveDialogModel::<Conf>::name()), &model.name),
            add_filter: iterate!(&model.settings.filters),
            set_create_folders: model.settings.create_folders,
            set_cancel_label: Some(model.settings.cancel_label),
            set_accept_label: Some(model.settings.accept_label),
            set_modal: model.settings.is_modal,
            set_transient_for: parent!(parent_widgets.parent_window().as_ref()),
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
