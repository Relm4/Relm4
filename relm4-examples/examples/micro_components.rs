use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, send, AppUpdate, MicroComponent, MicroModel, MicroWidgets, Model, RelmApp, Sender,
    WidgetPlus, Widgets,
};

#[derive(Debug)]
enum NumberMsg {
    Increment,
    Decrement,
}

#[derive(Debug)]
struct NumberModel {
    num: u8,
}

impl MicroModel for NumberModel {
    type Msg = NumberMsg;
    type Widgets = NumberWidgets;
    type Data = ();

    fn update(&mut self, msg: NumberMsg, _data: &(), _sender: Sender<Self::Msg>) {
        match msg {
            NumberMsg::Increment => self.num = self.num.wrapping_add(1),
            NumberMsg::Decrement => self.num = self.num.wrapping_sub(1),
        }
    }
}

#[relm4::micro_widget]
#[derive(Debug)]
impl MicroWidgets<NumberModel> for NumberWidgets {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,
            append = &gtk::Label {
                set_label: watch!(&model.num.to_string()),
                set_margin_start: 10,
                set_margin_end: 10,
            },
            append = &gtk::Button {
                set_label: "Increment",
                set_hexpand: true,
                connect_clicked(sender) => move |_| send!(sender, NumberMsg::Increment),
            },
            append = &gtk::Button {
                set_label: "Decrement",
                set_hexpand: true,
                connect_clicked(sender) => move |_| send!(sender, NumberMsg::Decrement),
            },
        }
    }
}

#[derive(Default)]
struct AppModel {
    counter: u8,
    numbers: Vec<MicroComponent<NumberModel>>,
}

enum AppMsg {
    Increment,
    Decrement,
    AddNumber,
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

                // Also increment the numbers.
                for num in &self.numbers {
                    let mut model = num.model_mut().unwrap();
                    model.num = model.num.wrapping_add(1);

                    // Make sure to drop the mutable reference before updating the view
                    drop(model);
                    num.update_view().unwrap();
                }
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
            AppMsg::AddNumber => {
                self.numbers
                    .push(MicroComponent::new(NumberModel { num: self.counter }, ()));
            }
        }
        true
    }
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                },
                append = &gtk::Button::with_label("Decrement") {
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Decrement);
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                },
                append = &gtk::Button::with_label("Add element") {
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::AddNumber);
                    },
                },
            },
        }
    }

    fn pre_view() {
        for num in &model.numbers {
            if !num.is_connected() {
                self.main_box.append(num.root_widget());
            }
        }
    }
}

fn main() {
    let model = AppModel::default();
    let app = RelmApp::new(model);
    app.run();
}
