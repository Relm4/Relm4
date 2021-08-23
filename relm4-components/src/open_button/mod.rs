use gtk::prelude::{BoxExt, ButtonExt, PopoverExt, WidgetExt};
use relm4::factory::{DynamicIndex, Factory, FactoryVecDeque};
use relm4::{send, ComponentUpdate, Components, Model, RelmComponent, Widgets};

use crate::open_dialog::{OpenDialogModel, OpenDialogMsg, OpenDialogParent, OpenDialogSettings};
use crate::ParentWindow;

use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

mod factory;

use factory::FileListItem;

#[tracker::track]
pub struct OpenButtonModel {
    #[do_not_track]
    config: OpenButtonSettings,
    #[do_not_track]
    dialog_config: OpenDialogSettings,
    #[do_not_track]
    recent_files: Option<FactoryVecDeque<FileListItem>>,
    initialized: bool,
    #[do_not_track]
    reset_popover: bool,
}

pub struct OpenButtonSettings {
    /// Text of the open button.
    pub text: &'static str,
    /// Path to a file where recent files should be stored.
    /// This list is updated fully automatically.
    pub recently_opened_files: Option<&'static str>,
    /// Maximum amount of recent files to store.
    /// This is only used if a path for storing the recently opened files was set.
    pub max_recent_files: usize,
}

#[doc(hidden)]
pub enum OpenButtonMsg {
    Open(PathBuf),
    OpenRecent(Rc<DynamicIndex>),
    ShowDialog,
    Ignore,
}

impl Model for OpenButtonModel {
    type Msg = OpenButtonMsg;
    type Widgets = OpenButtonWidgets;
    type Components = OpenButtonComponents;
}

pub trait OpenButtonParent: Model
where
    Self::Widgets: ParentWindow,
{
    /// Returns a configuration for the open dialog.
    fn dialog_config(&self) -> OpenDialogSettings;
    /// Returns a configuration for the open button.
    fn open_button_config(&self) -> OpenButtonSettings;

    /// Returns the message the button will send to the parent
    /// with the file path the user wants to open.
    fn open_msg(path: PathBuf) -> Self::Msg;
}

impl<ParentModel> ComponentUpdate<ParentModel> for OpenButtonModel
where
    ParentModel: Model + OpenButtonParent,
    ParentModel::Widgets: ParentWindow,
{
    fn init_model(parent_model: &ParentModel) -> Self {
        OpenButtonModel {
            config: parent_model.open_button_config(),
            dialog_config: parent_model.dialog_config(),
            recent_files: None,
            initialized: false,
            reset_popover: false,
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<ParentModel::Msg>,
    ) {
        self.reset();
        self.reset_popover = false;

        if !self.initialized {
            self.set_initialized(true);
            if let Some(path) = self.config.recently_opened_files {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(path)
                    .expect("Couldn't create nor open recent files file");
                let mut entries = String::new();
                match file.read_to_string(&mut entries) {
                    Ok(_) => {
                        let mut list = FactoryVecDeque::new();
                        for file_name in entries.split('\n') {
                            if !file_name.is_empty() {
                                list.push_back(FileListItem {
                                    path: PathBuf::from(file_name),
                                });
                            }
                        }
                        self.recent_files = Some(list);
                    }
                    Err(err) => {
                        log::warn!("{}", err);
                    }
                }
            }
        }
        match msg {
            OpenButtonMsg::ShowDialog => {
                components.dialog.send(OpenDialogMsg::Open).unwrap();
            }
            OpenButtonMsg::Open(path) => {
                send!(parent_sender, ParentModel::open_msg(path.clone()));
                self.reset_popover = true;
                if let Some(recent_files) = &mut self.recent_files {
                    if let Some(index) = recent_files.iter().position(|item| item.path == path) {
                        let data = recent_files.remove(index).unwrap();
                        recent_files.push_front(data);
                    } else {
                        recent_files.push_front(FileListItem { path });
                    }
                    if recent_files.len() > self.config.max_recent_files {
                        recent_files.pop_back();
                    }
                    let file_content: String = recent_files
                        .iter()
                        .map(|item| {
                            format!(
                                "{}\n",
                                item.path.to_str().expect("Couldn't convert path to string")
                            )
                        })
                        .collect();
                    std::fs::write(self.config.recently_opened_files.unwrap(), &file_content)
                        .expect("Couldn't write to recent files list");
                }
            }
            OpenButtonMsg::OpenRecent(index) => {
                if let Some(item) = self
                    .recent_files
                    .as_ref()
                    .unwrap()
                    .get(index.current_index())
                {
                    send!(sender, OpenButtonMsg::Open(PathBuf::from(&item.path)));
                }
            }
            OpenButtonMsg::Ignore => (),
        }
    }
}

impl OpenDialogParent for OpenButtonModel {
    fn dialog_config(&self) -> OpenDialogSettings {
        self.dialog_config.clone()
    }

    fn open_msg(path: PathBuf) -> OpenButtonMsg {
        OpenButtonMsg::Open(path)
    }
}

impl ParentWindow for OpenButtonWidgets {
    fn parent_window(&self) -> Option<gtk::Window> {
        self.parent_window.clone()
    }
}

pub struct OpenButtonComponents {
    dialog: RelmComponent<OpenDialogModel, OpenButtonModel>,
}

impl Components<OpenButtonModel> for OpenButtonComponents {
    fn init_components(
        parent_model: &OpenButtonModel,
        parent_widget: &OpenButtonWidgets,
        parent_sender: relm4::Sender<OpenButtonMsg>,
    ) -> Self {
        OpenButtonComponents {
            dialog: RelmComponent::new(parent_model, parent_widget, parent_sender),
        }
    }
}

#[relm4_macros::widget(pub)]
impl<ParentModel> Widgets<OpenButtonModel, ParentModel> for OpenButtonWidgets
where
    ParentModel: Model + OpenButtonParent,
    ParentModel::Widgets: ParentWindow,
{
    view! {
        open_box = gtk::Box {
            add_css_class: "linked",
            append = &gtk::Button {
                set_label: model.config.text,
                connect_clicked(sender) => move |_| {
                    send!(sender, OpenButtonMsg::ShowDialog);
                }
            },
        }
    }

    additional_fields! {
        parent_window: Option<gtk::Window>,
        view: Option<gtk::Box>,
        popover: Option<gtk::Popover>,
        scroll_window: Option<gtk::ScrolledWindow>
    }

    fn post_init() {
        let parent_window = parent_widgets.parent_window();

        let (view, popover, scroll_window) = if model.config.recently_opened_files.is_some() {
            let drop_down_button = gtk::MenuButton::new();
            let popover = gtk::Popover::new();
            let window = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .min_content_width(100)
                .min_content_height(100)
                .max_content_height(300)
                .build();
            let view = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .vexpand(true)
                .hexpand(true)
                .build();
            view.add_css_class("linked");

            open_box.append(&drop_down_button);
            drop_down_button.set_popover(Some(&popover));
            window.set_child(Some(&view));
            popover.set_child(Some(&window));

            send!(sender, OpenButtonMsg::Ignore);

            (Some(view), Some(popover), Some(window))
        } else {
            (None, None, None)
        };
    }

    fn manual_view() {
        if let Some(model) = &model.recent_files {
            model.generate(self.view.as_ref().expect("Box wasn't generated"), sender);
        }

        if model.reset_popover {
            if let Some(popover) = &self.popover {
                popover.popdown();
            }
        }

        if let Some(window) = &self.scroll_window {
            window.emit_scroll_child(gtk::ScrollType::Start, false);
        }
    }
}
