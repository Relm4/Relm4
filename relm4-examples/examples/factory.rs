use gtk::glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::factory::{Factory, FactoryPrototype, FactoryVec};
use relm4::{send, AppUpdate, Model, RelmApp, WidgetPlus, Widgets};

#[derive(Debug)]
enum AppMsg {
    Add,
    Remove,
    Clicked(usize),
}

struct Counter {
    value: u8,
}

struct AppModel {
    counters: FactoryVec<Counter>,
    created_counters: u8,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Add => {
                self.counters.push(Counter {
                    value: self.created_counters,
                });
                self.created_counters += 1;
            }
            AppMsg::Remove => {
                self.counters.pop();
            }
            AppMsg::Clicked(index) => {
                if let Some(counter) = self.counters.get_mut(index) {
                    counter.value = counter.value.wrapping_sub(1);
                }
            }
        }
        true
    }
}

struct FactoryWidgets {
    button: gtk::Button,
}

impl FactoryPrototype for Counter {
    type Factory = FactoryVec<Self>;
    type Widgets = FactoryWidgets;
    type Root = gtk::Button;
    type View = gtk::Box;
    type Msg = AppMsg;

    fn generate(&self, index: &usize, sender: Sender<AppMsg>) -> FactoryWidgets {
        let button = gtk::Button::with_label(&self.value.to_string());
        let index = *index;
        button.connect_clicked(move |_| {
            sender.send(AppMsg::Clicked(index)).unwrap();
        });

        FactoryWidgets { button }
    }

    fn position(&self, _index: &usize) {}

    fn update(&self, _index: &usize, widgets: &FactoryWidgets) {
        widgets.button.set_label(&self.value.to_string());
    }

    fn get_root(widgets: &FactoryWidgets) -> &gtk::Button {
        &widgets.button
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_default_width: 300,
            set_default_height: 200,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                append = &gtk::Button {
                    set_label: "Add",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Add);
                    }
                },
                append = &gtk::Button {
                    set_label: "Remove",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Remove);
                    }
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,
                    set_spacing: 5,
                    factory!(model.counters),
                }
            }
        }
    }
}

fn main() {
    let model = AppModel {
        counters: FactoryVec::new(),
        created_counters: 0,
    };

    let relm = RelmApp::new(model);
    relm.run();
}
