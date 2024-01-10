use gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::factory::{FactoryComponent, FactoryHashMap, FactorySender};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
struct Counter {
    name: String,
    value: u8,
}

#[derive(Debug)]
enum CounterOutput {}

#[relm4::factory]
impl FactoryComponent for Counter {
    type Init = u8;
    type Input = ();
    type Output = CounterOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Stack;
    type Index = String;

    view! {
        #[root]
        root = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_spacing: 10,
            set_margin_all: 12,

            #[name(label)]
            gtk::Label {
                set_use_markup: true,
                #[watch]
                set_label: &format!("<b>Counter value: {}</b>", self.value),
                set_width_chars: 3,
            },

        },
        #[local_ref]
        returned_widget -> gtk::StackPage {
            set_name: &self.name,
            set_title: &self.name,
        }
    }

    fn init_model(value: Self::Init, index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            name: index.clone(),
            value,
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        println!("Counter with value {} was destroyed", self.value);
    }
}

struct App {
    created_widgets: u8,
    counters: FactoryHashMap<String, Counter>,
    entry_buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
enum AppMsg {
    UpdateView,
    AddCounter,
    Increment(String),
    Decrement(String),
    RemoveCounter(String),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Factory example"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::StackSwitcher {
                    set_stack: Some(counter_stack),
                },

                gtk::Entry {
                    set_buffer: &model.entry_buffer,
                    connect_activate => AppMsg::AddCounter,
                },

                #[name(add_button)]
                gtk::Button {
                    set_label: "Increment",
                    #[watch]
                    set_sensitive: counter_stack.visible_child().is_some(),
                    connect_clicked[sender, counter_stack] => move |_| {
                        if let Some(name) = counter_stack.visible_child_name() {
                            sender.input(AppMsg::Increment(name.into()));
                        }
                    },
                },

                #[name(remove_button)]
                gtk::Button {
                    set_label: "Decrement",
                    #[watch]
                    set_sensitive: counter_stack.visible_child().is_some(),
                    connect_clicked[sender, counter_stack] => move |_| {
                        if let Some(name) = counter_stack.visible_child_name() {
                            sender.input(AppMsg::Decrement(name.into()));
                        }
                    },
                },

                gtk::Button {
                    set_label: "Remove counter",
                    #[watch]
                    set_sensitive: counter_stack.visible_child().is_some(),
                    connect_clicked[sender, counter_stack] => move |_| {
                        if let Some(name) = counter_stack.visible_child_name() {
                            sender.input(AppMsg::RemoveCounter(name.into()));
                        }
                    },
                },

                #[local_ref]
                counter_stack -> gtk::Stack {
                    connect_visible_child_notify => AppMsg::UpdateView,
                }
            }
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let counters = FactoryHashMap::builder().launch_default().detach();

        let model = App {
            created_widgets: counter,
            counters,
            entry_buffer: gtk::EntryBuffer::default(),
        };

        let counter_stack = model.counters.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::AddCounter => {
                let index = self.entry_buffer.text().to_string();
                if !index.is_empty() {
                    self.counters.insert(index, self.created_widgets);
                    self.created_widgets = self.created_widgets.wrapping_add(1);
                }
            }
            AppMsg::Increment(key) => {
                let mut elem = self.counters.get_mut(&key).unwrap();
                elem.value = elem.value.saturating_add(1);
            }
            AppMsg::Decrement(key) => {
                let mut elem = self.counters.get_mut(&key).unwrap();
                elem.value = elem.value.saturating_sub(1);
            }
            AppMsg::RemoveCounter(key) => {
                self.counters.remove(&key);
            }
            AppMsg::UpdateView => (),
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.factory");
    app.run::<App>(0);
}
