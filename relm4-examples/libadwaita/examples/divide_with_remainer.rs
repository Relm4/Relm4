use adw::traits::ApplicationWindowExt;
use gtk::{
    prelude::{BoxExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt}};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

#[derive(Default)]
struct AppModel {
    dividend: Option<i32>,
    divisor: Option<i32>,
    result: Option<i32>,
    remainder: Option<i32>,
    error: Option<String>,
}

enum AppMsg {
    Calc,
    SetDividend(Option<i32>),
    SetDivisor(Option<i32>),
    SetError(Option<String>),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Calc => {
                if let Some(dividend) = self.dividend {
                    if let Some(divisor) = self.divisor {
                        if divisor == 0 {
                            self.error = Some("Divisor must be not 0".to_string());
                            return true;
                        }
                        self.result = Some(dividend / divisor);
                        self.remainder = Some(dividend % divisor);
                    } else {
                        self.error = Some("Please provide divisor".to_string());
                    }
                } else {
                    self.error = Some("Please provide dividend".to_string());
                }
            }
            AppMsg::SetDividend(v) => self.dividend = v,

            AppMsg::SetDivisor(v) => {
                if v == Some(0) {
                    self.error = Some("Divisor must be not 0".to_string());
                    self.divisor = None;
                }
                self.divisor = v;
            }
            AppMsg::SetError(v) => self.error = v,
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

            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &adw::HeaderBar {
                    set_title_widget = Some(&gtk::Label) {
                        set_label: "Division with Remainder",
                    }
                },
                append = &gtk::Box {
                    set_margin_all: 5,
                    set_spacing: 5,
                    append : remainder = &gtk::Entry {
                        connect_buffer_notify(sender) => move |entry| {
                            let value = entry.buffer().text().parse::<i32>();
                                if let Ok(v) = value {
                                    send!(sender, AppMsg::SetDividend(Some(v)));
                                    send!(sender, AppMsg::SetError(None))
                                } else {
                                    send!(sender, AppMsg::SetDividend(None));
                                    send!(sender, AppMsg::SetError(Some("Dividend is not a number".to_string())))
                                }
                        },
                    },
                    append = &gtk::Label {
                        set_label: ":",
                    },
                    append = &gtk::Entry {
                        connect_activate(sender) => move |entry| {
                          //  remainder.clone().buffer().text();
                            let value = entry.buffer().text().parse::<i32>();
                                if let Ok(v) = value {
                                    send!(sender, AppMsg::SetDivisor(Some(v)));
                                    send!(sender, AppMsg::SetError(None));
                                    send!(sender, AppMsg::Calc);
                                } else {
                                    send!(sender, AppMsg::SetDivisor(None));
                                    send!(sender, AppMsg::SetError(Some("Divisor is not a number".to_string())))
                                }
                            send!(sender, AppMsg::Calc);
                        },
                    },
                    append = &gtk::Label {
                        set_label: watch! { &*if let Some(r) = model.result {
                            format!(" = {} R{}", r, model.remainder.unwrap())
                        } else {" =".to_string()} }
                    }
                },
                append = &gtk::Label {
                    set_label: watch! {&*if let Some(v) = model.error.clone() {
                        v
                    } else {
                        "".to_string()
                    }}
                }
            }
        }
    }
}

fn main() {
    let model = AppModel::default();
    let app = RelmApp::new(model);
    app.run();
}
