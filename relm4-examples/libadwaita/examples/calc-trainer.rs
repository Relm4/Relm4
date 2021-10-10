use adw::traits::ApplicationWindowExt;
use gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    ToggleButtonExt, WidgetExt,
};
use gtk::EntryBuffer;
use rand::Rng;
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

enum PracticeMode {
    Plus,
    Minus,
    Multiply,
}

enum TaskType {
    ValueValueEntry,
    ValueEntryValue,
    EntryValueValue,
}

struct AppModel {
    mode: PracticeMode,
    task_type: TaskType,
    range: u32,
    display_task_1: String,
    display_task_2: String,
    correct_value: u32,
    feedback: String,
}

impl AppModel {
    fn calculate_task(&mut self) {
        match self.mode {
            PracticeMode::Plus => match self.task_type {
                TaskType::ValueValueEntry => {
                    self.correct_value = rand::thread_rng().gen_range(1..=self.range);
                    let v1 = rand::thread_rng().gen_range(1..=self.correct_value);
                    let v2 = self.correct_value - v1;
                    self.display_task_1 = format!("<big>{} + {} = </big>", v1, v2);
                }
                TaskType::ValueEntryValue => {
                    let result = rand::thread_rng().gen_range(1..=self.range);
                    let v1 = rand::thread_rng().gen_range(1..=result);
                    self.correct_value = result - v1;
                    self.display_task_1 = format!("<big>{} + </big>", v1);
                    self.display_task_2 = format!("<big> = {}</big>", result);
                }
                TaskType::EntryValueValue => {
                    let result = rand::thread_rng().gen_range(1..=self.range);
                    self.correct_value = rand::thread_rng().gen_range(1..=result);
                    let v2 = result - self.correct_value;
                    self.display_task_2 = format!("<big> + {} = {}</big>", v2, result);
                }
            },
            PracticeMode::Minus => match self.task_type {
                TaskType::ValueValueEntry => {
                    let v1 = rand::thread_rng().gen_range(1..=self.range);
                    let v2 = rand::thread_rng().gen_range(0..=v1);
                    self.correct_value = v1 - v2;
                    self.display_task_1 = format!("<big>{} - {} = </big>", v1, v2);
                }
                TaskType::ValueEntryValue => {
                    let v1 = rand::thread_rng().gen_range(1..=self.range);
                    self.correct_value = rand::thread_rng().gen_range(0..=v1);
                    let result = v1 - self.correct_value;
                    self.display_task_1 = format!("<big>{} - </big>", v1);
                    self.display_task_2 = format!("<big> = {}</big>", result);
                }
                TaskType::EntryValueValue => {
                    self.correct_value = rand::thread_rng().gen_range(1..=self.range);
                    let v2 = rand::thread_rng().gen_range(0..=self.correct_value);
                    let result = self.correct_value - v2;
                    self.display_task_2 = format!("<big> - {} = {}</big>", v2, result);
                }
            },
            PracticeMode::Multiply => match self.task_type {
                TaskType::ValueValueEntry => {
                    let v1 = rand::thread_rng().gen_range(0..=self.range);
                    let v2 = rand::thread_rng().gen_range(0..=self.range);
                    self.correct_value = v1 * v2;
                    self.display_task_1 = format!("<big>{} ∙ {} = </big>", v1, v2);
                }
                TaskType::ValueEntryValue => {
                    let v1 = rand::thread_rng().gen_range(0..=self.range);
                    self.correct_value = rand::thread_rng().gen_range(0..=self.range);
                    let result = v1 * self.correct_value;
                    self.display_task_1 = format!("<big>{} ∙ </big>", v1);
                    self.display_task_2 = format!("<big> = {}</big>", result);
                }
                TaskType::EntryValueValue => {
                    self.correct_value = rand::thread_rng().gen_range(0..=self.range);
                    let v2 = rand::thread_rng().gen_range(0..=self.range);
                    let result = self.correct_value * v2;
                    self.display_task_2 = format!("<big> ∙ {} = {}</big>", v2, result);
                }
            },
        }
    }

    fn pick_random_task_type(&mut self) {
        let task_type = rand::thread_rng().gen_range(0..3);

        match task_type {
            0 => self.task_type = TaskType::ValueValueEntry,
            1 => self.task_type = TaskType::ValueEntryValue,
            _ => self.task_type = TaskType::EntryValueValue,
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
                self.calculate_task();
            }
            AppMsg::Multiply => {
                self.mode = PracticeMode::Multiply;
                self.calculate_task();
            }
            AppMsg::Minus => {
                self.mode = PracticeMode::Minus;
                self.calculate_task();
            }
            AppMsg::MaxValue(v) => {
                self.range = v;
                self.calculate_task();
            }
            AppMsg::Entry(v) => {
                if self.correct_value == v as u32 {
                    self.feedback = "<big>😀 That was right!! 💓</big>".to_string();
                    self.pick_random_task_type();
                    self.calculate_task();
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

fn application_window() -> adw::ApplicationWindow {
    adw::ApplicationWindow::builder().build()
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = application_window() -> adw::ApplicationWindow {
            set_default_width: 300,
            set_default_height: 200,
            set_resizable: false,

            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    set_title_widget = Some(&adw::WindowTitle::new("Practice mental arithmetic!",
                        "Challenge yourself with math")) {
                    }

                },
                append = &gtk::Label {
                    set_text: "Calculation Type:"
                },
                append = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_margin_all: 5,
                    set_spacing: 5,
                    append: plus_button = &gtk::ToggleButton {
                        set_label: "+",
                        set_active: true,
                        connect_toggled(sender) => move |b| {
                            if b.is_active() {
                                send!(sender, AppMsg::Plus);
                            }
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "-",
                        set_group: Some(&plus_button),
                        connect_toggled(sender) => move |b| {
                            if b.is_active() {
                                send!(sender, AppMsg::Minus);
                            }
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "∙",
                        set_group: Some(&plus_button),
                        connect_toggled(sender) => move |b| {
                            if b.is_active() {
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
                append : stack = &gtk::Stack {
                    add_child: value_value_entry = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 5,
                        set_spacing: 5,
                        set_halign: gtk::Align::Center,
                        append = &gtk::Label {
                            set_markup: watch! { model.display_task_1.as_str() },
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
                        }
                    },
                    add_child: value_entry_value = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 5,
                        set_spacing: 5,
                        set_halign: gtk::Align::Center,
                        append = &gtk::Label {
                            set_markup: watch! { model.display_task_1.as_str() },
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
                        append = &gtk::Label {
                            set_markup: watch! { model.display_task_2.as_str() },
                        },
                    },
                    add_child: entry_value_value = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_all: 5,
                        set_spacing: 5,
                        set_halign: gtk::Align::Center,
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
                        append = &gtk::Label {
                            set_markup: watch! { model.display_task_2.as_str() },
                        },
                    },
                },
                append: feedback = &gtk::Label {
                    set_markup: watch!( &model.feedback ),
                }
            }
        }
    }

    fn manual_view() {
        match model.task_type {
            TaskType::EntryValueValue => self.stack.set_visible_child(&self.entry_value_value),
            TaskType::ValueEntryValue => self.stack.set_visible_child(&self.value_entry_value),
            TaskType::ValueValueEntry => self.stack.set_visible_child(&self.value_value_entry),
        }
    }
}

fn main() {
    let mut model = AppModel {
        mode: PracticeMode::Plus,
        range: 10,
        display_task_1: "".to_string(),
        display_task_2: "".to_string(),
        correct_value: 0,
        feedback: "<big>Welcome to the mental math trainer!</big>".to_string(),
        task_type: TaskType::ValueValueEntry,
    };
    model.calculate_task();

    let app = RelmApp::new(model);
    app.run();
}
