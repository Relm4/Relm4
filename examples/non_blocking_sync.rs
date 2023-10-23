use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt};

struct App {
    counter: u8,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = ();
    type Output = ();
    type CommandOutput = Msg;
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Async Counter"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    // Messages are fully async, no blocking!
                    connect_clicked[sender] => move |_| {
                        sender.spawn_oneshot_command(|| {
                            std::thread::sleep(Duration::from_secs(1));
                            Msg::Increment
                        })
                    },
                },

                gtk::Button::with_label("Decrement") {
                    connect_clicked[sender] => move |_| {
                        sender.spawn_oneshot_command(|| {
                            std::thread::sleep(Duration::from_secs(1));
                            Msg::Decrement
                        })
                    },
                },

                gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App { counter: 0 };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            Msg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.non_blocking_async");
    app.run::<App>(());
}
