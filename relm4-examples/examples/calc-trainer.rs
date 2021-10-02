use adw::traits::ApplicationWindowExt;
use gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt, ToggleButtonExt, WidgetExt
};
use gtk::EntryBuffer;
use rand::Rng;
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

enum PracticeMode {
    Plus,
    Minus,
    Multiply,
}

struct AppModel {
    mode: PracticeMode,
    range: u32,
    display_task: String,
    correct_value: u32,
    feedback: String,
}

impl AppModel {
    fn calc_taks(&mut self) {
        match self.mode {
            PracticeMode::Plus => {
                self.correct_value = rand::thread_rng().gen_range(1..=self.range);
                let v1 = rand::thread_rng().gen_range(1..=self.correct_value);
                let v2 = self.correct_value - v1;
                self.display_task = format!("<big>{} + {} = </big>", v1, v2);
            }
            PracticeMode::Minus => {
                let v1 = rand::thread_rng().gen_range(1..=self.range);
                let v2 = rand::thread_rng().gen_range(0..=v1);
                self.correct_value = v1 - v2;
                self.display_task = format!("<big>{} - {} = </big>", v1, v2);
            }
            PracticeMode::Multiply => {
                let v = rand::thread_rng().gen_range(0..=self.range);
                let v1 = rand::thread_rng().gen_range(1..=v);
                let v2 = v / v1;
                println!("v: {}, v1: {}, v2: {}", v, v1, v2);
                self.correct_value = v1 * v2;
                self.display_task = format!("<big>{} ∙ {} = </big>", v1, v2);
            }
        }
    }
}

enum AppMsg {
    Plus,
    Minus,
    Multiply,
    MaxValue(u32),
    Entry(i32),
    EntryError,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Plus => {
                self.mode = PracticeMode::Plus;
                self.calc_taks();
            }
            AppMsg::Multiply => {
                self.mode = PracticeMode::Multiply;
                self.calc_taks();
            }
            AppMsg::Minus => {
                self.mode = PracticeMode::Minus;
                self.calc_taks();
            }
            AppMsg::MaxValue(v) => {
                self.range = v;
                self.calc_taks();
            }
            AppMsg::Entry(v) => {
                if self.correct_value == v as u32 {
                    self.feedback = "<big>😀 That was right!! 💓</big>".to_string();
                    self.calc_taks();
                } else {
                    self.feedback = "<big>😕 Unfortunately wrong. 😓</big>".to_string();
                }
            }
            AppMsg::EntryError => {
                self.feedback = "<big>Please enter a valid number.</big>".to_string();
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = adw::ApplicationWindow {
            GtkWindowExt::set_default_width: 300,
            GtkWindowExt::set_default_height: 200,

            ApplicationWindowExt::set_child: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                },
                append = &adw::HeaderBar {
                    set_title_widget = Some(&gtk::Label) {
                        set_label: "Practice mental arithmetic!",
                    }
                },
                append = &gtk::Label {
                    set_text: "Calculation Type:"
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_valign: gtk::Align::Center,
                    set_margin_all: 5,
                    set_spacing: 5,
                    append: plus_button = &gtk::ToggleButton {
                        set_label: "+",
                        set_active: true,
                        connect_toggled(sender) => move |b| {
                            if b.is_active() == true {
                                send!(sender, AppMsg::Plus);
                            }
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "-",
                        set_group: Some(&plus_button),
                        connect_toggled(sender) => move |b| {
                            if b.is_active() == true {
                                send!(sender, AppMsg::Minus);
                            }
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "∙",
                        set_group: Some(&plus_button),
                        connect_toggled(sender) => move |b| {
                            if b.is_active() == true {
                                send!(sender, AppMsg::Multiply);
                            }
                        },
                    },
                    append = &gtk::DropDown::from_strings(&["0-10", "0-20", "0-100", "0-1000"]) {
                        connect_selected_notify(sender) => move |dd| {
                            match dd.selected() {
                                0 => send!(sender, AppMsg::MaxValue(10)),
                                1 => send!(sender, AppMsg::MaxValue(20)),
                                2 => send!(sender, AppMsg::MaxValue(100)),
                                3 => send!(sender, AppMsg::MaxValue(1000)),
                                _ => {},
                            }
                        }
                    }
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 5,
                    set_spacing: 5,
                    append = &gtk::Label {
                        set_markup: watch! { &model.display_task.as_str() },

                    },
                    append = &gtk::Entry {
                        connect_activate(sender) => move |entry| {
                            let value = entry.buffer().text().parse::<i32>();
                            if let Ok(v) = value {
                                send!(sender, AppMsg::Entry(v));
                            } else {
                                send!(sender, AppMsg::EntryError);
                            }
                            entry.set_buffer(&EntryBuffer::new(None));
                        }
                    },
                },
                append: feedback = &gtk::Label {
                    set_markup: watch!( &model.feedback ),
                }
            },
        }
    }
}

fn main() {
    let mut model = AppModel {
        mode: PracticeMode::Plus,
        range: 10,
        display_task: "".to_string(),
        correct_value: 0,
        feedback: "<big>Welcome to the mental math trainer!</big>".to_string(),
    };
    model.calc_taks();

    let app = RelmApp::new(model);
    app.run();
}
