use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

#[derive(Debug)]
enum AppMsg {
    AddCounters,
    Clicked(DynamicIndex),
}

struct Counter {
    value: u8,
}

struct App {
    counters: FactoryVecDeque<Counter>,
    created_counters: u8,
    // stores entered values
    entry: gtk::EntryBuffer,
}

#[relm4::factory]
impl FactoryComponent for Counter {
    type Init = u8;
    type Input = ();
    type Output = AppMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            #[watch]
            set_label: &self.value.to_string(),
            connect_clicked[index] => move |_| {
                sender.output(AppMsg::Clicked(index.clone())).unwrap();
            },
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { value }
    }
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Entry example"),
            set_default_size: (300, 200),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Entry {
                    set_buffer: &model.entry,
                    set_tooltip_text: Some("How many counters shall be added/removed?"),
                    connect_activate => AppMsg::AddCounters,
                },

                #[local]
                factory_box -> gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,
                    set_spacing: 5,
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let factory_box = gtk::Box::default();

        let counters = FactoryVecDeque::builder()
            .launch(factory_box.clone())
            .forward(sender.input_sender(), |output| output);

        let model = App {
            counters,
            created_counters: 0,
            entry: gtk::EntryBuffer::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::AddCounters => {
                let text = self.entry.text();
                if let Ok(v) = text.parse::<i32>() {
                    let mut guard = self.counters.guard();
                    if v.is_positive() {
                        // add as many counters as user entered
                        for _ in 0..v {
                            guard.push_back(self.created_counters);
                            self.created_counters += 1;
                        }
                    } else if v.is_negative() {
                        // remove counters
                        for _ in v..0 {
                            guard.pop_front();
                        }
                    }

                    // clearing the entry value clears the entry widget
                    self.entry.set_text("");
                }
            }
            AppMsg::Clicked(index) => {
                if let Some(counter) = self.counters.guard().get_mut(index.current_index()) {
                    counter.value = counter.value.wrapping_sub(1);
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.entry");
    app.run::<App>(());
}
