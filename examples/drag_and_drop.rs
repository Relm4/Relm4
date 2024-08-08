use gtk::prelude::{ButtonExt, GtkWindowExt, OrientableExt, StaticType, ToValue, WidgetExt};
use gtk::{self, gdk, glib};
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

// we need a glib type that can be send through drag and drop
// basic rust types like String, u8, i8 and others are ready to go
// see: https://gtk-rs.org/gtk-rs-core/stable/0.18/docs/glib/value/index.html
//
// for custom structs this can be accomplished by registering it as
// a boxed type
// see: https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/subclass/index.html#example-for-registering-a-boxed-type-for-a-rust-struct
// or wrapped with BoxedAnyObject but it has the downside of not as easily
// beeing distinguishable in conjunction with the set_types method
// see: https://docs.rs/glib/latest/glib/struct.BoxedAnyObject.html

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "BlueKey")] // the name for the glib Type, needs to be unique
struct BlueKey;

#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "LockPick")]
struct LockPick;

struct AppModel {
    status: String,
}

#[derive(Debug)]
enum AppIn {
    NewStatus(String),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppIn;
    type Output = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            status: String::from("Drag the key or lock pick to a door"),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        gtk::Window {
            set_title: Some("Drag and Drop Expample"),
            set_default_width: 300,
            set_default_height: 100,

            // outer box
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                // display status message
                gtk::Label {
                    set_margin_all: 10,
                    #[watch]
                    set_label: &model.status,
                },

                gtk::Label {
                    set_margin_all: 10,
                    set_label: "The blue key will only unlock the blue door\nThe pick can open both",
                },

                // first horizontal box
                gtk::CenterBox {
                    // contains a image and label and serves as DropTarget
                    #[wrap(Some)]
                    set_start_widget = &gtk::Button {
                        gtk::Box {
                            set_margin_all: 10,

                            gtk::Image {
                                set_icon_name: Some("locked"),
                            },
                            gtk::Label {
                                set_label: "Blue Door",
                            }
                        },

                        // drops will be accepted on all widgets in Button
                        add_controller = gtk::DropTarget {
                            // this DropTarget will accept DragAction::MOVE and
                            // drops with the types BlueKey and LockPick
                            set_actions: gdk::DragAction::MOVE,
                            set_types: &[BlueKey::static_type(),
                                        LockPick::static_type()],
                            connect_drop[sender] => move |_widget, value, _x, _y| {
                                if let Ok(_key) = value.get::<BlueKey>() {
                                    // handle your dropping of BlueKey here
                                    sender.input(AppIn::NewStatus(
                                        String::from("Blue Door unlocked with Blue Key")
                                    ));
                                    return true;
                                }
                                if let Ok(_pick) = value.get::<LockPick>() {
                                    // handle your dropping of LockPick here
                                    sender.input(AppIn::NewStatus(
                                        String::from("Blue Door unlocked with Lock Pick")
                                    ));
                                    return true;
                                }
                                false
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_end_widget = &gtk::Button {
                        set_label: "Blue Key",

                        // define the Button as a DragSource
                        add_controller = gtk::DragSource {
                            set_actions: gdk::DragAction::MOVE,
                            // which value will be send when dropping
                            set_content: Some(&gdk::ContentProvider::for_value(&BlueKey.to_value())),
                        }
                    }
                },

                // second horizontal box
                gtk::CenterBox {
                    // contains image and label and serves as a DropTarget
                    #[wrap(Some)]
                    set_start_widget = &gtk::Button {
                        gtk::Box {
                            set_margin_all: 10,

                            gtk::Image {
                                set_icon_name: Some("locked"),
                            },
                            gtk::Label {
                                set_label: "Red Door",
                            }
                        },

                        add_controller = gtk::DropTarget {
                            set_actions: gdk::DragAction::MOVE,
                            set_types: &[LockPick::static_type()],
                            connect_drop[sender] => move |_widget, _value, _x, _y| {
                                // there is only the pick lock to unlock the door
                                sender.input(AppIn::NewStatus(
                                   String::from("Red Door unlocked with Lock Pick")
                                ));
                                true
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_end_widget = &gtk::Button {
                        set_label: "Lock Pick",
                        add_controller = gtk::DragSource {
                            set_actions: gdk::DragAction::MOVE,
                            set_content: Some(&gdk::ContentProvider::for_value(&LockPick.to_value())),
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppIn::NewStatus(status) => {
                self.status = status;
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.drag-and-drop");
    app.run::<AppModel>(());
}
