use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Default)]
struct App {
    counter: u8,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Popup example"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::MenuButton {
                    set_label: "Pop up!",
                    set_direction: gtk::ArrowType::Right,

                    #[wrap(Some)]
                    set_popover: popover = &gtk::Popover {
                        set_position: gtk::PositionType::Right,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,

                            gtk::Button {
                                set_label: "Increment counter",
                                connect_clicked => Msg::Increment,
                            },

                            gtk::Button {
                                set_label: "Decrement counter",
                                connect_clicked => Msg::Decrement,
                            },
                        },
                    },
                },
                gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { counter: 0 };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
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
    let app = RelmApp::new("relm4.example.popover");
    app.run::<App>(());
}
