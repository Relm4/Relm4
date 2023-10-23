use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

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
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked => AppMsg::Increment,
                },
                append = &gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => AppMsg::Decrement,
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    #[watch(skip_init)]
                    set_label: &format!("Counter: {}{value}", model.counter),
                    #[track(skip_init, test)]
                    set_label: value,
                }
            },
        }
    }

    fn pre_view() {
        // These values are not available in init
        // so we can make sure "skip_init" works
        // by accessing the variables.
        let test = true;
        let value = "value";
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
