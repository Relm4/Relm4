use gtk::prelude::{BoxExt, ButtonExt, GridExt, GtkWindowExt, OrientableExt};
use relm4::factory::positions::GridPosition;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque, Position};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

#[derive(Debug)]
struct Counter {
    value: u8,
}

#[derive(Debug)]
enum CounterMsg {
    Increment,
    Decrement,
}

#[derive(Debug)]
enum CounterOutput {
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

struct CounterWidgets {
    label: gtk::Label,
}

impl Position<GridPosition, DynamicIndex> for Counter {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();
        let x = index % 5;
        let y = index / 5;
        GridPosition {
            column: y as i32,
            row: x as i32,
            width: 1,
            height: 1,
        }
    }
}

impl FactoryComponent for Counter {
    type Init = u8;
    type Input = CounterMsg;
    type Output = CounterOutput;
    type CommandOutput = ();
    type Root = gtk::Box;
    type Widgets = CounterWidgets;
    type ParentWidget = gtk::Grid;
    type Index = DynamicIndex;

    fn init_root(&self) -> Self::Root {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
            }
        }
        root
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { value }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                #[name(label)]
                gtk::Label {
                    set_label: &self.value.to_string(),
                    set_width_chars: 3,
                },

                gtk::Button {
                    set_label: "+",
                    connect_clicked => CounterMsg::Increment,
                },

                gtk::Button {
                    set_label: "-",
                    connect_clicked => CounterMsg::Decrement,
                },

                gtk::Button {
                    set_label: "Up",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::MoveUp(index.clone())).unwrap();
                    }
                },

                gtk::Button {
                    set_label: "Down",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::MoveDown(index.clone())).unwrap();
                    }
                },

                gtk::Button {
                    set_label: "To Start",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(CounterOutput::SendFront(index.clone())).unwrap();
                    }
                }
            }
        }

        CounterWidgets { label }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            CounterMsg::Increment => {
                self.value = self.value.wrapping_add(1);
            }
            CounterMsg::Decrement => {
                self.value = self.value.wrapping_sub(1);
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: FactorySender<Self>) {
        widgets.label.set_label(&self.value.to_string());
    }
}

struct App {
    created_widgets: u8,
    counters: FactoryVecDeque<Counter>,
}

#[derive(Debug)]
enum AppMsg {
    AddCounter,
    RemoveCounter,
    SendFront(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Grid factory example"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Add counter",
                    connect_clicked => AppMsg::AddCounter,
                },

                gtk::Button {
                    set_label: "Remove counter",
                    connect_clicked => AppMsg::RemoveCounter,
                },

                #[local_ref]
                counter_grid -> gtk::Grid {
                    set_orientation: gtk::Orientation::Vertical,
                    set_column_spacing: 15,
                    set_row_spacing: 5,
                }
            }
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let counters =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CounterOutput::SendFront(index) => AppMsg::SendFront(index),
                    CounterOutput::MoveUp(index) => AppMsg::MoveUp(index),
                    CounterOutput::MoveDown(index) => AppMsg::MoveDown(index),
                });

        let model = App {
            created_widgets: counter,
            counters,
        };

        let counter_grid = model.counters.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        let mut counters_guard = self.counters.guard();

        match msg {
            AppMsg::AddCounter => {
                counters_guard.push_back(self.created_widgets);
                self.created_widgets = self.created_widgets.wrapping_add(1);
            }
            AppMsg::RemoveCounter => {
                counters_guard.pop_back();
            }
            AppMsg::SendFront(index) => {
                counters_guard.move_front(index.current_index());
            }
            AppMsg::MoveDown(index) => {
                let index = index.current_index();
                let new_index = index + 1;
                // Already at the end?
                if new_index < counters_guard.len() {
                    counters_guard.move_to(index, new_index);
                }
            }
            AppMsg::MoveUp(index) => {
                let index = index.current_index();
                // Already at the start?
                if index != 0 {
                    counters_guard.move_to(index, index - 1);
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.grid_factory");
    app.run::<App>(0);
}
