use gtk::glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::adw::prelude::WidgetExt;
use relm4::adw::traits::AdwApplicationWindowExt;
use relm4::factory::positions::GridPosition;
use relm4::factory::{FactoryPrototype, FactoryVec};
use relm4::{adw, gtk, send, AppUpdate, Model, RelmApp, WidgetPlus, Widgets};
use sudoku::Sudoku;

#[derive(Debug)]
enum AppMsg {
    Solve,
    Clear,
    Update(usize, u8),
}

#[derive(Clone, Debug)]
struct Number {
    value: u8,
}

struct AppModel {
    values: FactoryVec<Number>,
}

impl AppModel {
    fn new() -> Self {
        Self {
            values: FactoryVec::from_vec(vec![Number { value: 0 }; 81]),
        }
    }

    fn to_line_string(&self) -> String {
        let mut res = String::new();
        for v in self.values.iter() {
            let string = match v.value {
                0 => String::from("."),
                v @ 1..=9 => format!("{v}"),
                v => panic!("{v} is not defined"),
            };
            res.push_str(string.as_str())
        }
        res
    }

    fn from_string(&mut self, input: String) {
        for (i, v) in input.chars().enumerate() {
            if v != '.' {
                let value = v.to_string().parse().unwrap();
                if let Some(v) = self.values.get_mut(i) {
                    v.value = value;
                }
            }
        }
    }
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Solve => {
                let content = self.to_line_string();
                let sudoku = Sudoku::from_str_line(content.as_str()).unwrap();

                if let Some(solution) = sudoku.solve_one() {
                    self.from_string(solution.to_string());
                }
            }
            AppMsg::Clear => {
                for index in 0..self.values.len() {
                    if let Some(v) = self.values.get_mut(index) {
                        v.value = 0;
                    }
                }
            }
            AppMsg::Update(index, value) => {
                if let Some(v) = self.values.get_mut(index) {
                    v.value = value;
                }
            }
        }
        true
    }
}

#[derive(Debug)]
struct FactoryWidgets {
    value: gtk::DropDown,
}

impl FactoryPrototype for Number {
    type Factory = FactoryVec<Self>;
    type Widgets = FactoryWidgets;
    type Root = gtk::DropDown;
    type View = gtk::Grid;
    type Msg = AppMsg;

    fn init_view(&self, index: &usize, sender: Sender<AppMsg>) -> FactoryWidgets {
        let dropdown =
            gtk::DropDown::from_strings(&[" ", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);

        if index % 3 == 0 {
            dropdown.set_margin_start(5);
        }
        if (index / 9) % 3 == 0 {
            dropdown.set_margin_top(5);
        }

        // When available:
        // dropdown.set_show_arrow(false);
        let index = *index;
        dropdown.connect_selected_notify(move |d| {
            sender
                .send(AppMsg::Update(index, d.selected() as u8))
                .unwrap();
        });

        FactoryWidgets { value: dropdown }
    }

    fn position(&self, index: &usize) -> GridPosition {
        let index = *index as i32;

        let row = index / 9;
        let column = index % 9;

        GridPosition {
            column,
            row,
            width: 1,
            height: 1,
        }
    }

    fn view(&self, _index: &usize, widgets: &FactoryWidgets) {
        widgets.value.set_selected(self.value as u32);
    }

    fn root_widget(widgets: &FactoryWidgets) -> &gtk::DropDown {
        &widgets.value
    }
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        adw::ApplicationWindow {
            set_default_width: 100,
            set_default_height: 100,
            set_resizable: false,
            set_content = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                append = &adw::HeaderBar {
                    set_title_widget = Some(&adw::WindowTitle::new("Sudoku Solver",
                        "Beat your neighbor in solving Sudoku puzzles")) {
                    },
                    pack_start =&gtk::Button {
                        set_label: "Clear",
                        connect_clicked(sender) => move |_| {
                            send!(sender, AppMsg::Clear);
                        }
                    },
                    pack_end = &gtk::Button {
                        set_label: "Solve",
                        connect_clicked(sender) => move |_| {
                            send!(sender, AppMsg::Solve);
                        }
                    },
                },
                append = &gtk::Grid {
                    set_margin_all: 5,
                    factory!(model.values),
                }
            }
        }
    }
}

fn main() {
    let model = AppModel::new();

    let relm = RelmApp::new(model);
    relm.run();
}
