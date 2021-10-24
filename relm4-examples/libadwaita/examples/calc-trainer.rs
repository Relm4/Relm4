use core::time;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread;

use adw::traits::ApplicationWindowExt;
use gtk::prelude::{
    BoxExt, ButtonExt, CheckButtonExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    PopoverExt, ToggleButtonExt, WidgetExt,
};
use gtk::EntryBuffer;
use rand::prelude::SliceRandom;
use rand::Rng;
use relm4::{
    send, AppUpdate, Components, MessageHandler, Model, RelmApp, RelmMsgHandler, Sender,
    WidgetPlus, Widgets,
};

#[derive(Clone)]
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
    plus: bool,
    minus: bool,
    multiply: bool,
    mode: PracticeMode,
    task_type: TaskType,
    range: u32,
    display_task_1: String,
    display_task_2: String,
    correct_value: u32,
    feedback: String,
    timer: Option<u32>,
    timer_init_value: u32,
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
                    let v1 = rand::thread_rng().gen_range(1..=self.range);
                    let v2 = rand::thread_rng().gen_range(0..=self.range);
                    self.correct_value = v1 * v2;
                    self.display_task_1 = format!("<big>{} ∙ {} = </big>", v1, v2);
                }
                TaskType::ValueEntryValue => {
                    let v1 = rand::thread_rng().gen_range(1..=self.range);
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
        let mut v = Vec::new();
        if self.minus {
            v.push(PracticeMode::Minus);
        }
        if self.plus {
            v.push(PracticeMode::Plus);
        }
        if self.multiply {
            v.push(PracticeMode::Multiply);
        }

        if !v.is_empty() {
            self.mode = v.choose(&mut rand::thread_rng()).unwrap().clone();
        } else {
            return;
        }

        let task_type = rand::thread_rng().gen_range(0..3);
        match task_type {
            0 => self.task_type = TaskType::ValueValueEntry,
            1 => self.task_type = TaskType::ValueEntryValue,
            _ => self.task_type = TaskType::EntryValueValue,
        }
    }
}

enum AppMsg {
    Plus(bool),
    Minus(bool),
    Multiply(bool),
    MaxValue(u32),
    Entry(i32),
    EntryError,
    SetTimer(u32),
    StartTimer,
    CountDown,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Plus(v) => {
                if self.minus || self.multiply {
                    self.plus = v;
                }
            }
            AppMsg::Multiply(v) => {
                if self.minus || self.plus {
                    self.multiply = v;
                }
            }
            AppMsg::Minus(v) => {
                if self.plus || self.multiply {
                    self.minus = v;
                }
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
            AppMsg::SetTimer(v) => {
                self.timer_init_value = v;
            }
            AppMsg::StartTimer => {
                // if timer is running a press on the button will reset button
                // otherwise initialize the timer with the selected init value
                components.timer_handler.send(TimerMsg::StartStopTimer);
                if self.timer.is_some() {
                    self.timer = None;
                } else {
                    self.timer = Some(self.timer_init_value);
                }
            }
            AppMsg::CountDown => {
                if let Some(t) = &mut self.timer {
                    *t -= 1;

                    if *t == 0 {
                        self.timer = None;
                        // stop timer
                        components.timer_handler.send(TimerMsg::StartStopTimer);
                    }
                }
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
            set_default_width: 350,
            set_default_height: 200,
            set_resizable: false,

            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    set_title_widget = Some(&adw::WindowTitle::new("Practice mental arithmetic!",
                        "Challenge yourself with math")) {
                    },
                    pack_start = &adw::SplitButton {
                        set_label: watch!({
                            &*if let Some(t) = model.timer {
                                format!("{}s", t)
                            } else {
                                "Start".to_string()
                            }
                        }),
                        connect_clicked(sender) => move |_| {
                            send!(sender, AppMsg::StartTimer)
                        },
                        set_popover: popover = Some(&gtk::Popover) {
                            set_child = Some(&gtk::Box) {
                                set_orientation: gtk::Orientation::Vertical,
                                append: timer = &gtk::CheckButton::with_label("30s") {
                                    connect_toggled(sender) => move |b| {
                                        if b.is_active() {
                                            send!(sender, AppMsg::SetTimer(30))
                                        }
                                    }
                                },
                                append = &gtk::CheckButton::with_label("60s") {
                                    set_group: Some(&timer),
                                    connect_toggled(sender) => move |b| {
                                        if b.is_active() {
                                            send!(sender, AppMsg::SetTimer(60))
                                        }
                                    }
                                },
                                append = &gtk::CheckButton::with_label("180s") {
                                    set_group: Some(&timer),
                                    connect_toggled(sender) => move |b| {
                                        if b.is_active() {
                                            send!(sender, AppMsg::SetTimer(180))
                                        }
                                    }
                                }
                            }
                        }
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
                    append = &gtk::ToggleButton {
                        set_label: "+",
                        set_active: watch!(model.plus),
                        connect_toggled(sender) => move |b| {
                            send!(sender, AppMsg::Plus(b.is_active()));
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "-",
                        set_active: watch!(model.minus),
                        connect_toggled(sender) => move |b| {
                            send!(sender, AppMsg::Minus(b.is_active()));
                        },
                    },
                    append = &gtk::ToggleButton {
                        set_label: "∙",
                        set_active: watch!(model.multiply),
                        connect_toggled(sender) => move |b| {
                            send!(sender, AppMsg::Multiply(b.is_active()));
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

    fn post_init() {
        // Set radio button initial to 30s
        timer.set_active(true);
    }
}

struct TimerHandler {
    _rt: (),
    sender: std::sync::mpsc::Sender<TimerMsg>,
}

#[derive(Debug)]
enum TimerMsg {
    StartStopTimer,
}

impl MessageHandler<AppModel> for TimerHandler {
    type Msg = TimerMsg;
    type Sender = std::sync::mpsc::Sender<TimerMsg>;

    fn init(_parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            loop {
                // Wait for start message
                let _ = receiver.recv();
                // loop as long channel is empty
                while receiver.try_recv().err() == Some(TryRecvError::Empty) {
                    thread::sleep(time::Duration::from_secs(1));
                    send!(parent_sender, AppMsg::CountDown);
                }
            }
        });

        TimerHandler { _rt: (), sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}

struct AppComponents {
    timer_handler: RelmMsgHandler<TimerHandler, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(
        parent_model: &AppModel,
        _parent_widget: &AppWidgets,
        parent_sender: Sender<AppMsg>,
    ) -> Self {
        AppComponents {
            timer_handler: RelmMsgHandler::new(parent_model, parent_sender),
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
        plus: true,
        minus: false,
        multiply: false,
        timer_init_value: 30,
        timer: None,
    };

    model.calculate_task();

    let app = RelmApp::new(model);
    app.run();
}
