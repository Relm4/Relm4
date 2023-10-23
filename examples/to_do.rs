use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

#[derive(Debug)]
struct Task {
    name: String,
    completed: bool,
}

#[derive(Debug)]
enum TaskInput {
    Toggle(bool),
}

#[derive(Debug)]
enum TaskOutput {
    Delete(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent for Task {
    type Init = String;
    type Input = TaskInput;
    type Output = TaskOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,

            gtk::CheckButton {
                set_active: false,
                set_margin_all: 12,
                connect_toggled[sender] => move |checkbox| {
                    sender.input(TaskInput::Toggle(checkbox.is_active()));
                }
            },

            #[name(label)]
            gtk::Label {
                set_label: &self.name,
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_margin_all: 12,
            },

            gtk::Button {
                set_icon_name: "edit-delete",
                set_margin_all: 12,

                connect_clicked[sender, index] => move |_| {
                    sender.output(TaskOutput::Delete(index.clone())).unwrap();
                }
            }
        }
    }

    fn pre_view() {
        let attrs = widgets.label.attributes().unwrap_or_default();
        attrs.change(gtk::pango::AttrInt::new_strikethrough(self.completed));
        widgets.label.set_attributes(Some(&attrs));
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name,
            completed: false,
        }
    }
}

#[derive(Debug)]
enum AppMsg {
    DeleteEntry(DynamicIndex),
    AddEntry(String),
}

struct App {
    tasks: FactoryVecDeque<Task>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_width_request: 360,
            set_title: Some("To-Do"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 6,

                gtk::Entry {
                    connect_activate[sender] => move |entry| {
                        let buffer = entry.buffer();
                        sender.input(AppMsg::AddEntry(buffer.text().into()));
                        buffer.delete_text(0, None);
                    }
                },

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_height: 360,
                    set_vexpand: true,

                    #[local_ref]
                    task_list_box -> gtk::ListBox {}
                }
            }

        }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::DeleteEntry(index) => {
                self.tasks.guard().remove(index.current_index());
            }
            AppMsg::AddEntry(name) => {
                self.tasks.guard().push_back(name);
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let tasks =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |output| match output {
                    TaskOutput::Delete(index) => AppMsg::DeleteEntry(index),
                });

        let model = App { tasks };

        let task_list_box = model.tasks.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.to_do");
    app.run::<App>(());
}
