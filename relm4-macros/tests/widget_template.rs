use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, WidgetTemplate};

#[relm4_macros::widget_template]
impl WidgetTemplate for CustomBox {
    type Init = i32;
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: init,
            set_spacing: 5,

            #[name = "child_label"]
            gtk::Label {
                set_label: "This is a test",
            }
        }
    }
}

#[relm4_macros::widget_template]
impl WidgetTemplate for CustomWindow {
    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            #[name = "test_label"]
            gtk::Label {}
        }
    }
}

#[derive(Default)]
struct App {
    counter: u8,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[template]
        CustomWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            // Make sure that using template children works
            #[template_child]
            test_label {
                set_label: "It works",
            },

            #[template]
            CustomBox(5) {
                gtk::Button {
                    set_label: "Increment",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Increment);
                    },
                },
                gtk::Button {
                    set_label: "Decrement",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Decrement);
                    },
                },
                #[template_child]
                child_label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },
                #[template]
                CustomBox(5) {
                    #[template_child]
                    child_label {
                        set_margin_all: 5,
                        #[track = "true"]
                        set_label: &format!("Alternative counter : {}", model.counter),
                    }
                },
            },
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { counter };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}
