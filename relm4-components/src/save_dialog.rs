use relm4::{impl_model, ComponentUpdate, Model, Sender, send};
use gtk::prelude::{FileChooserExt, NativeDialogExt, FileExt};

use std::path::PathBuf;

pub struct SaveDialogSettings {
    pub cancel_label: String,
    pub accept_label: String,
    pub create_folders: bool,
    pub filters: Vec<gtk::FileFilter>,
}

#[tracker::track]
pub struct SaveDialogModel {
    #[no_eq]
    settings: SaveDialogSettings,
    suggestion: Option<String>,
    is_active: bool,
    invalid_input: bool,
    name: String,
}

pub enum SaveDialogMsg {
    Save,
    SaveAs(String),
    Accept(PathBuf),
    InvalidInput,
    Cancel,
}

impl_model!(SaveDialogModel, SaveDialogMsg);

pub trait SaveDialogParent: Model {
    fn dialog_config(&self) -> SaveDialogSettings;
    fn save_msg(path: PathBuf) -> Self::Msg;
}

impl<ParentModel> ComponentUpdate<ParentModel> for SaveDialogModel
where
    ParentModel: SaveDialogParent,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        SaveDialogModel {
            settings: parent_model.dialog_config(),
            is_active: false,
            invalid_input: false,
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
                self.invalid_input = false;
            }
            SaveDialogMsg::SaveAs(name) => {
                self.is_active = true;
                self.invalid_input = false;
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
impl relm4::RelmWidgets for SaveDialogWidgets {
    type Model = SaveDialogModel;

    view! {
        gtk::FileChooserNative {
            set_action: gtk::FileChooserAction::Save,
            set_visible: watch!(model.is_active),
            set_current_name: track!(model.changed(SaveDialogModel::name()), &model.name),
            set_create_folders: model.settings.create_folders,
            set_cancel_label: Some(&model.settings.cancel_label),
            set_accept_label: Some(&model.settings.accept_label),
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
