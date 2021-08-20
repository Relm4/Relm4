use gtk::prelude::{BoxExt, ButtonExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

struct AppModel {
    counter: u8,
    classes: Vec<&'static str>,
}

enum AppMsg {
    Increment,
    Decrement,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
        true
    }
}

fn new_label() -> gtk::Label {
    gtk::Label::new(Some("test"))
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            gtk::prelude::GtkWindowExt::set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                    add_class_name: iterate!(&model.classes),
                },
                append = &gtk::Button::new() {
                    set_label: track!(false, "Decrement"),
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Decrement);
                    },
                },
                append = &new_label() -> gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                },
                append = &gtk::Grid {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_row_spacing: 10,
                    set_column_spacing: 10,
                    set_column_homogeneous: true,
                    attach(1, 1, 1, 1) = &gtk::Label {
                        set_label: "grid test 1",
                    },
                    attach(1, 2, 1, 1) = &gtk::Label {
                        set_label: "grid test 2",
                    },
                    attach(2, 1, 1, 1) = &gtk::Label {
                        set_label: "grid test 3",
                    },
                    attach(2, 2, 1, 1) = &gtk::Label {
                        set_label: "grid test 4",
                    },
                }
            },
        }
    }

    additional_fields! {
        test: u8,
    }

    fn pre_init() {
        let test = 0;
        println!("Pre init!");
    }

    fn post_init() {
        relm4::set_global_css(b".first { color: green; } .second { border: 1px solid orange; }");
        println!("Post init!");
    }

    fn manual_view() {
        println!("Manual view!");
    }
}

fn main() {
    let model = AppModel {
        counter: 0,
        classes: vec!["first", "second"],
    };
    let app = RelmApp::new(model);
    app.run();
}
