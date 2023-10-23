use gtk::prelude::*;
use relm4::factory::{FactoryVecDeque, FactoryView};
use relm4::prelude::*;
use relm4::Worker;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_components::save_dialog::{
    SaveDialog, SaveDialogMsg, SaveDialogResponse, SaveDialogSettings,
};
use relm4_icons::icon_name;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_SPACING: i32 = 5;
const XALIGN_CENTER: f32 = 0.5;
const CSS_CLASS_DESTRUCTIVE_ACTION: &str = "destructive-action";

/// The view.
///
/// Any state held within the view components is private and self-contained, and is solely used to
/// render information on screen. Any exchange of data with external components is done via events.
/// As a result the view has no knowledge of the document - indeed it is possible to remove the
/// document and replace it with a different implementation without the view knowing.
#[derive(Debug)]
struct Task {
    name: String,
    tags: FactoryVecDeque<Tag>,
}

#[derive(Debug)]
enum TaskInput {
    // events of Task object
    ChangedName(String),

    // events broadcast downwards to nested Tag objects
    AddedTag(String),
    DeletedTag(usize),
}

#[derive(Debug)]
enum TaskOutput {
    // events of Task object
    Name(DynamicIndex, String),
    Delete(DynamicIndex),

    // events bubbled up from nested Tag objects
    AddTag(DynamicIndex, String),
    DeleteTag(DynamicIndex, DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for Task {
    type Init = ();
    type Input = TaskInput;
    type Output = TaskOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: DEFAULT_SPACING,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: DEFAULT_SPACING,

                #[name(label)]
                gtk::Entry {
                    #[watch]
                    set_text: &self.name,
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,

                    connect_activate[sender, index] => move |entry| {
                        // activate means 'enter' was pressed, so user is done editing
                        let new_name: String = entry.text().into();
                        sender.output(TaskOutput::Name(index.clone(), new_name)).unwrap();
                    }
                },

                gtk::Button {
                    set_icon_name: icon_name::DELETE_FILLED,
                    set_tooltip: "Delete Task",

                    connect_clicked[sender, index] => move |_| {
                        sender.output(TaskOutput::Delete(index.clone())).unwrap();
                    }
                },
            },

            gtk::Box {
                set_spacing: DEFAULT_SPACING,
                set_orientation: gtk::Orientation::Horizontal,

                gtk::MenuButton {
                    set_icon_name: icon_name::TAG_OUTLINE_ADD,
                    set_tooltip: "Add Tag",

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: DEFAULT_SPACING,

                            gtk::Button {
                                set_label: "#home",

                                connect_clicked[sender, index] => move |_| {
                                    sender.output(TaskOutput::AddTag(index.clone(), "#home".into())).unwrap();
                                }
                            },

                            gtk::Button {
                                set_label: "#work",

                                connect_clicked[sender, index] => move |_| {
                                    sender.output(TaskOutput::AddTag(index.clone(), "#work".into())).unwrap();
                                }
                            }
                        }
                    }
                },

                #[local_ref]
                tag_list_box -> gtk::Box {
                    set_spacing: DEFAULT_SPACING,
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            TaskInput::ChangedName(name) => {
                self.name = name;
            }
            TaskInput::AddedTag(name) => {
                self.tags.guard().push_back(name);
            }
            TaskInput::DeletedTag(index) => {
                self.tags.guard().remove(index);
            }
        }
    }

    fn init_widgets(
        &mut self,
        index: &Self::Index,
        root: &Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let tag_list_box = self.tags.widget();

        let widgets = view_output!();

        widgets
    }

    fn init_model(_name: Self::Init, index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let task_index = index.clone();

        let tags = FactoryVecDeque::builder().launch_default().forward(
            sender.output_sender(),
            move |output| match output {
                TagOutput::Delete(tag_index) => {
                    TaskOutput::DeleteTag(task_index.clone(), tag_index)
                }
            },
        );

        Self {
            name: "".into(),
            tags,
        }
    }
}

#[derive(Debug)]
struct Tag {
    name: String,
}

#[derive(Debug)]
enum TagInput {}

#[derive(Debug)]
enum TagOutput {
    Delete(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for Tag {
    type Init = String;
    type Input = TagInput;
    type Output = TagOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::MenuButton {
            #[watch]
            set_label: &self.name,

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                gtk::Button {
                    set_label: "Delete",

                    connect_clicked[sender, index] => move |_| {
                        sender.output(TagOutput::Delete(index.clone())).unwrap();
                    }
                }
            }
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name }
    }
}

/// The document is a headless component which holds and manages the data model.
/// It receives input events FROM the App to update the data model.
/// When updates to the model occur, it sends output events TO the App.
///
/// The document's interface is just input and output events. As a result you have a lot of freedom
/// in how you choose to store the data model within the component, which backing store you use
/// (such as the file system, a database, or a Web API), and how you synchronise to the backing
/// store (e.g. manual save/load control, auto-saving on each change, batching up changes before
/// syncing, and so on).
struct Document {
    /// The application data model.
    /// In this case we have just stored the whole thing in memory because our requirements are
    /// simple. In a real app you might choose a more elaborate approach.
    model: Model,
}

#[derive(Default, Serialize, Deserialize)]
struct TagModel {
    name: String,
}
#[derive(Default, Serialize, Deserialize)]
struct TaskModel {
    name: String,
    tags: Vec<TagModel>,
}
#[derive(Default, Serialize, Deserialize)]
struct Model {
    tasks: Vec<TaskModel>,
}

#[derive(Debug)]
enum DocumentInput {
    // extra operations on the document itself (in this case, related to file I/O)
    Open(PathBuf),
    Save(PathBuf),

    // events related to the model that the document stores
    Clear,
    AddTask,
    DeleteTask(DynamicIndex),
    ChangeTaskName(DynamicIndex, String),
    AddTag(DynamicIndex, String),
    DeleteTag(DynamicIndex, DynamicIndex),
}

#[derive(Debug)]
enum DocumentOutput {
    Cleared,
    AddedTask,
    DeletedTask(usize),
    ChangedTaskName(usize, String),
    AddedTag(usize, String),
    DeletedTag(usize, usize),
}

impl Worker for Document {
    type Init = ();
    type Input = DocumentInput;
    type Output = DocumentOutput;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        let model = Model::default();
        Self { model }
    }

    fn update(&mut self, input: DocumentInput, sender: ComponentSender<Self>) {
        match input {
            DocumentInput::Save(path) => {
                println!("Save as JSON to {:?}", path);

                // TODO in a real app you would report any errors from saving the document
                if let Ok(json) = serde_json::to_string(&self.model) {
                    std::fs::write(path, json).unwrap();
                }
            }
            DocumentInput::Open(path) => {
                println!("Open tasks document at {:?}", path);

                if let Ok(json) = std::fs::read_to_string(path) {
                    if let Ok(new_model) = serde_json::from_str(&json) {
                        // update the data model
                        self.model = new_model;

                        // refresh the view from the data model
                        let _ = sender.output(DocumentOutput::Cleared);

                        for (task_index, task) in self.model.tasks.iter().enumerate() {
                            let _ = sender.output(DocumentOutput::AddedTask);

                            let task_name = task.name.clone();
                            let _ = sender
                                .output(DocumentOutput::ChangedTaskName(task_index, task_name));

                            for tag in &task.tags {
                                let tag_name = tag.name.clone();
                                let _ =
                                    sender.output(DocumentOutput::AddedTag(task_index, tag_name));
                            }
                        }
                    }
                }
            }
            DocumentInput::Clear => {
                self.model.tasks.clear();

                let _ = sender.output(DocumentOutput::Cleared);
            }
            DocumentInput::AddTask => {
                self.model.tasks.push(TaskModel::default());

                let _ = sender.output(DocumentOutput::AddedTask);
            }
            DocumentInput::DeleteTask(index) => {
                self.model.tasks.remove(index.current_index());

                let _ = sender.output(DocumentOutput::DeletedTask(index.current_index()));
            }
            DocumentInput::ChangeTaskName(index, name) => {
                if let Some(task) = self.model.tasks.get_mut(index.current_index()) {
                    task.name = name.clone();
                }

                // We don't technically need to send an event, because gtk::Entry updates itself
                // this is just to make the example consistent.
                let _ = sender.output(DocumentOutput::ChangedTaskName(index.current_index(), name));
            }
            DocumentInput::AddTag(task_index, name) => {
                if let Some(task) = self.model.tasks.get_mut(task_index.current_index()) {
                    task.tags.push(TagModel { name: name.clone() })
                }

                let _ = sender.output(DocumentOutput::AddedTag(task_index.current_index(), name));
            }
            DocumentInput::DeleteTag(task_index, tag_index) => {
                if let Some(task) = self.model.tasks.get_mut(task_index.current_index()) {
                    task.tags.remove(tag_index.current_index());
                }

                let _ = sender.output(DocumentOutput::DeletedTag(
                    task_index.current_index(),
                    tag_index.current_index(),
                ));
            }
        }
    }
}

/// The App is at the top level.
/// It acts as a bridge between the view and the document, forwarding events between them.
struct App {
    view: FactoryVecDeque<Task>,
    document: Controller<Document>,
    save_dialog: Controller<SaveDialog>,
    open_dialog: Controller<OpenDialog>,
}

#[derive(Debug)]
enum AppInput {
    Clear,
    Cleared,

    AddTask,
    AddedTask,

    DeleteTask(DynamicIndex),
    DeletedTask(usize),

    ChangeTaskName(DynamicIndex, String),
    ChangedTaskName(usize, String),

    AddTag(DynamicIndex, String),
    AddedTag(usize, String),

    DeleteTag(DynamicIndex, DynamicIndex),
    DeletedTag(usize, usize),

    // No-op event for when load/save dialogs result in Cancel
    None,
    Open,
    OpenResponse(PathBuf),
    Save,
    SaveResponse(PathBuf),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_width_request: 360,
            set_title: Some("Tasks"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
                    set_show_title_buttons: false,

                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                        set_text: ""
                    },

                    pack_start = &gtk::Button {
                        set_icon_name: icon_name::PLUS,
                        set_tooltip: "Add Task",

                        connect_clicked[sender] => move |_| {
                            sender.input(AppInput::AddTask);
                        }
                    },

                    pack_end = &gtk::Button {
                        set_label: "Save",
                        connect_clicked => AppInput::Save,
                    },
                    pack_end = &gtk::Button {
                        set_label: "Open",
                        connect_clicked => AppInput::Open,
                    },
                },

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_height: 360,
                    set_vexpand: true,

                    #[local_ref]
                    task_list_box -> gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                    }
                },

                gtk::Box {
                    set_hexpand: true,
                    set_spacing: DEFAULT_SPACING,
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Label {
                        set_text: "Press Enter after editing task names",
                        set_hexpand: true,
                        set_xalign: XALIGN_CENTER,
                    },

                    gtk::Button {
                        set_icon_name: icon_name::DELETE_FILLED,
                        set_tooltip: "Delete All Tasks",
                        add_css_class: CSS_CLASS_DESTRUCTIVE_ACTION,

                        connect_clicked[sender] => move |_| {
                            sender.input(AppInput::Clear);
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self, msg: AppInput, _sender: ComponentSender<Self>) {
        match msg {
            AppInput::Clear => {
                self.document.emit(DocumentInput::Clear);
            }
            AppInput::Cleared => {
                self.view.guard().clear();
            }
            AppInput::AddTask => {
                self.document.emit(DocumentInput::AddTask);
            }
            AppInput::AddedTask => {
                self.view.guard().push_back(());
            }
            AppInput::DeleteTask(index) => {
                self.document.emit(DocumentInput::DeleteTask(index));
            }
            AppInput::DeletedTask(index) => {
                self.view.guard().remove(index);
            }
            AppInput::ChangeTaskName(index, name) => {
                self.document
                    .emit(DocumentInput::ChangeTaskName(index, name));
            }
            AppInput::ChangedTaskName(index, name) => {
                self.view.guard().send(index, TaskInput::ChangedName(name));
            }
            AppInput::AddTag(index, name) => {
                self.document.emit(DocumentInput::AddTag(index, name));
            }
            AppInput::AddedTag(index, name) => {
                self.view.guard().send(index, TaskInput::AddedTag(name));
            }
            AppInput::DeleteTag(task_index, tag_index) => {
                self.document
                    .emit(DocumentInput::DeleteTag(task_index, tag_index));
            }
            AppInput::DeletedTag(task_index, tag_index) => {
                self.view
                    .guard()
                    .send(task_index, TaskInput::DeletedTag(tag_index));
            }
            AppInput::None => {}
            AppInput::Save => {
                let name = "tasks.json".into();
                self.save_dialog.emit(SaveDialogMsg::SaveAs(name));
            }
            AppInput::SaveResponse(path) => {
                self.document.emit(DocumentInput::Save(path));
            }
            AppInput::Open => {
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            AppInput::OpenResponse(path) => {
                self.document.emit(DocumentInput::Open(path));
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let view =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    TaskOutput::Delete(index) => AppInput::DeleteTask(index),
                    TaskOutput::Name(index, name) => AppInput::ChangeTaskName(index, name),
                    TaskOutput::AddTag(index, name) => AppInput::AddTag(index, name),
                    TaskOutput::DeleteTag(task_index, tag_index) => {
                        AppInput::DeleteTag(task_index, tag_index)
                    }
                });

        let document =
            Document::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    DocumentOutput::Cleared => AppInput::Cleared,
                    DocumentOutput::DeletedTask(index) => AppInput::DeletedTask(index),
                    DocumentOutput::DeletedTag(task_index, tag_index) => {
                        AppInput::DeletedTag(task_index, tag_index)
                    }
                    DocumentOutput::AddedTask => AppInput::AddedTask,
                    DocumentOutput::AddedTag(index, name) => AppInput::AddedTag(index, name),
                    DocumentOutput::ChangedTaskName(index, name) => {
                        AppInput::ChangedTaskName(index, name)
                    }
                });

        let save_dialog = SaveDialog::builder()
            .transient_for_native(&root)
            .launch(SaveDialogSettings {
                create_folders: true,
                accept_label: "Save".into(),
                cancel_label: "Cancel".into(),
                is_modal: true,
                filters: tasks_filename_filters(),
            })
            .forward(sender.input_sender(), |response| match response {
                SaveDialogResponse::Accept(path) => AppInput::SaveResponse(path),
                SaveDialogResponse::Cancel => AppInput::None,
            });

        let open_dialog = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings {
                create_folders: false,
                folder_mode: false,
                cancel_label: "Cancel".into(),
                accept_label: "Open".into(),
                is_modal: true,
                filters: tasks_filename_filters(),
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => AppInput::OpenResponse(path),
                OpenDialogResponse::Cancel => AppInput::None,
            });

        let app = App {
            view,
            document,
            open_dialog,
            save_dialog,
        };

        let task_list_box = app.view.widget();
        let widgets = view_output!();

        ComponentParts {
            model: app,
            widgets,
        }
    }
}

fn tasks_filename_filters() -> Vec<gtk::FileFilter> {
    let filename_filter = gtk::FileFilter::default();
    filename_filter.set_name(Some("JSON (.json)"));
    filename_filter.add_suffix("json");

    vec![filename_filter]
}

///
/// This example demonstrates how to interact with persistent state in a Relm4 app, using Relm4's
/// one-way event-based data flow.
///
/// Events bubble up from view components to the top level, where they are forwarded down into the
/// document which persists them. When the persistent data model is changed, the document bubbles
/// events back up to the top level, where they are forwarded back down to the relevant view.
///
/// In an app with persistent state, view Components do not update their own view state as soon as
/// changes happen. Instead, an Inversion Of Control is used. They forward the change as an output
/// event (which goes up the view hierarchy), and trust that the persistent state store (the
/// document) will call them back with the relevant view state update later.
///
/// (This is the difference between e.g. the `AddTag` event (view -> document) which expresses the
/// change we would like to persist, and the `AddedTag` event (document -> view) which contains the
/// persistent change that has actually happened.)
///
fn main() {
    let app = RelmApp::new("relm4.example.state_management");

    relm4_icons::initialize_icons();

    app.run::<App>(());
}
