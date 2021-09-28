use gtk::prelude::{
    BoxExt, CheckButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::factory::{FactoryPrototype, FactoryVec};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

struct Task {
    name: String,
    completed: bool,
}

#[derive(Debug)]
struct TaskWidgets {
    label: gtk::Label,
    hbox: gtk::Box,
}

impl FactoryPrototype for Task {
    type View = gtk::ListBox;
    type Msg = AppMsg;
    type Factory = FactoryVec<Task>;
    type Widgets = TaskWidgets;
    type Root = gtk::Box;

    fn generate(&self, key: &usize, sender: Sender<Self::Msg>) -> Self::Widgets {
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();
        let checkbox = gtk::CheckButton::builder().active(false).build();
        let label = gtk::Label::new(Some(&self.name));

        assert!(!self.completed);

        checkbox.set_margin_all(12);
        label.set_margin_all(12);

        hbox.append(&checkbox);
        hbox.append(&label);

        let index = *key;
        checkbox.connect_toggled(move |checkbox| {
            send!(sender, AppMsg::SetCompleted((index, checkbox.is_active())));
        });

        TaskWidgets { label, hbox }
    }

    fn position(&self, _key: &usize) {}

    fn update(&self, _key: &usize, widgets: &Self::Widgets) {
        let attrs = widgets.label.attributes().unwrap_or_default();
        attrs.change(gtk::pango::Attribute::new_strikethrough(self.completed));
        widgets.label.set_attributes(Some(&attrs));
    }

    fn get_root(widgets: &Self::Widgets) -> &Self::Root {
        &widgets.hbox
    }
}

enum AppMsg {
    SetCompleted((usize, bool)),
    AddEntry(String),
}

struct AppModel {
    tasks: FactoryVec<Task>,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::SetCompleted((index, completed)) => {
                if let Some(task) = self.tasks.get_mut(index) {
                    task.completed = completed;
                }
            }
            AppMsg::AddEntry(name) => {
                self.tasks.push(Task {
                    name,
                    completed: false,
                });
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_width_request: 360,
            set_title: Some("To-Do"),

            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 6,

                append = &gtk::Entry {
                    connect_activate(sender) => move |entry| {
                        let buffer = entry.buffer();
                        send!(sender, AppMsg::AddEntry(buffer.text()));
                        buffer.delete_text(0, None);
                    }
                },

                append = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_min_content_height: 360,
                    set_vexpand: true,
                    set_child = Some(&gtk::ListBox) {
                        factory!(model.tasks),
                    }
                }
            }

        }
    }
}

fn main() {
    let model = AppModel {
        tasks: FactoryVec::new(),
    };
    let relm = RelmApp::new(model);
    relm.run();
}
