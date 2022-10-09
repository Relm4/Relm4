use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Default)]
struct AppModel {
    counter: u8,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Popup example"),
            set_default_width: 300,
            set_default_height: 100,

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
                                connect_clicked => AppMsg::Increment,
                            },

                            gtk::Button {
                                set_label: "Decrement counter",
                                connect_clicked => AppMsg::Decrement,
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
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { counter: 0 };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.popover");
    app.run::<AppModel>(());
}
