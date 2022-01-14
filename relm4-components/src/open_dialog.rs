//! Reusable and easily configurable open dialog component.
use gtk::prelude::{FileChooserExt, FileExt, NativeDialogExt};
use relm4::{gtk, send, ComponentUpdate, Model, Sender};

use std::marker::PhantomData;
use std::path::PathBuf;

use crate::ParentWindow;

#[derive(Clone, Debug)]
/// Configuration for the open dialog component
pub struct OpenDialogSettings {
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

/// Interface for creating OpenDialogSettings
pub trait OpenDialogConfig {
    /// Model from which OpenDialogSettings will be built
    type Model: Model;
    /// Builds configuration for OpenDialog
    fn open_dialog_config(model: &Self::Model) -> OpenDialogSettings;
}

#[tracker::track]
#[derive(Debug)]
/// Model of the open dialog component
pub struct OpenDialogModel<Conf> {
    #[do_not_track]
    settings: OpenDialogSettings,
    is_active: bool,
    #[do_not_track]
    _conf_provider: PhantomData<*const Conf>, //we don't own Conf, there is no instance of Conf
}

#[derive(Debug)]
/// Messages that can be sent to the open dialog component
pub enum OpenDialogMsg {
    /// Opens the dialog
    Open,
    #[doc(hidden)]
    Accept(PathBuf),
    #[doc(hidden)]
    InvalidInput,
    #[doc(hidden)]
    Cancel,
}

impl<Conf: OpenDialogConfig> Model for OpenDialogModel<Conf> {
    type Msg = OpenDialogMsg;
    type Widgets = OpenDialogWidgets;
    type Components = ();
}

/// Interface for the parent model
pub trait OpenDialogParent: Model
where
    Self::Widgets: ParentWindow,
{
    /// Tell the open dialog how to response if the user wants to open a file
    fn open_msg(path: PathBuf) -> Self::Msg;
}

impl<ParentModel, Conf> ComponentUpdate<ParentModel> for OpenDialogModel<Conf>
where
    ParentModel: OpenDialogParent,
    <ParentModel as relm4::Model>::Widgets: ParentWindow,
    Conf: OpenDialogConfig<Model = ParentModel>,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        Self {
            settings: Conf::open_dialog_config(parent_model),
            is_active: false,
            tracker: 0,
            _conf_provider: PhantomData,
        }
    }

    fn update(
        &mut self,
        msg: OpenDialogMsg,
        _components: &(),
        _sender: Sender<OpenDialogMsg>,
        parent_sender: Sender<ParentModel::Msg>,
    ) {
        self.reset();

        match msg {
            OpenDialogMsg::Open => {
                self.is_active = true;
            }
            OpenDialogMsg::Cancel => {
                self.is_active = false;
            }
            OpenDialogMsg::Accept(path) => {
                self.is_active = false;
                parent_sender.send(ParentModel::open_msg(path)).unwrap();
            }
            _ => (),
        }
    }
}

#[relm4::widget(visibility = pub)]
/// Widgets of the open dialog component
impl<ParentModel, Conf> relm4::Widgets<OpenDialogModel<Conf>, ParentModel> for OpenDialogWidgets
where
    ParentModel: Model,
    ParentModel::Widgets: ParentWindow,
    Conf: OpenDialogConfig,
{
    view! {
        gtk::FileChooserNative {
            set_action: gtk::FileChooserAction::Open,
            set_visible: watch!(model.is_active),
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
                                send!(sender, OpenDialogMsg::Accept(path));
                                return;
                            }
                        }
                        send!(sender, OpenDialogMsg::InvalidInput);
                    },
                    gtk::ResponseType::Cancel => {
                        send!(sender, OpenDialogMsg::Cancel)
                    },
                    _ => (),
                }
            },
        }
    }
}
