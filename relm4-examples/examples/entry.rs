use gtk::glib::Sender;
use gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::factory::{FactoryPrototype, FactoryVec};
use relm4::{gtk, send, AppUpdate, Model, RelmApp, WidgetPlus, Widgets};

#[derive(Debug)]
enum AppMsg {
    AddCounters,
    Clicked(usize),
}

struct Counter {
    value: u8,
}

struct AppModel {
    counters: FactoryVec<Counter>,
    created_counters: u8,
    // stores entered values
    entry: gtk::EntryBuffer,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::AddCounters => {
                let text = self.entry.text();
                if let Ok(v) = text.parse::<i32>() {
                    if v.is_positive() {
                        // add as many counters as user entered
                        for _ in 0..v {
                            self.counters.push(Counter {
                                value: self.created_counters,
                            });
                            self.created_counters += 1;
                        }
                    } else if v.is_negative() {
                        // remove counters
                        for _ in v..0 {
                            self.counters.pop();
                        }
                    }

                    // clearing the entry value clears the entry widget
                    self.entry.set_text("");
                }
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

#[derive(Debug)]
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
                append = &gtk::Entry {
                    set_buffer: &model.entry,
                    set_tooltip_text: Some("How many counters shall be added/removed?"),
                    connect_activate(sender) => move |_| {
                        send!(sender, AppMsg::AddCounters);
                    }
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,
                    set_spacing: 5,
                    factory!(model.counters),
                },
            }
        }
    }
}

fn main() {
    gtk::init().unwrap();

    let model = AppModel {
        counters: FactoryVec::new(),
        created_counters: 0,
        entry: gtk::EntryBuffer::new(None),
    };

    let relm = RelmApp::new(model);
    relm.run();
}
