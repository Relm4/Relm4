use gtk::prelude::{BoxExt, ButtonExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, send, AppUpdate, ComponentUpdate, Model, RelmApp, RelmComponent, Sender, WidgetPlus,
    Widgets, WidgetRef
};

#[tracker::track]
struct AppModel {
    counter: u8,
    classes: Vec<&'static str>,
    decrement: bool,
}

enum AppMsg {
    Increment,
    Decrement,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        msg: AppMsg,
        _components: &AppComponents,
        _sender: Sender<AppMsg>,
    ) -> bool {
        self.reset();
        match msg {
            AppMsg::Increment => {
                self.set_counter(self.counter.wrapping_add(1));
                self.decrement = false;
            }
            AppMsg::Decrement => {
                self.set_counter(self.counter.wrapping_sub(1));
                self.decrement = true;
            }
        }
        true
    }
}

struct ButtonMsg;

struct ButtonModel {}

impl Model for ButtonModel {
    type Msg = ButtonMsg;
    type Widgets = ButtonWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for ButtonModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        ButtonModel {}
    }

    fn update(
        &mut self,
        _msg: ButtonMsg,
        _components: &(),
        _sender: Sender<ButtonMsg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        println!("Message to component received!");
    }
}

#[relm4::widget]
impl Widgets<ButtonModel, AppModel> for ButtonWidgets {
    view! {
        gtk::Button {
            set_label: "ButtonComponent!",
        }
    }
}

#[derive(relm4::Components)]
pub struct AppComponents {
    button1: RelmComponent<ButtonModel, AppModel>,
    button2: RelmComponent<ButtonModel, AppModel>,
}

fn new_label() -> gtk::Label {
    gtk::Label::new(Some("test"))
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            gtk::prelude::GtkWindowExt::set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all?: Some(5),
                set_spacing: 5,

                append: &start_label,
                append: components.button1.root_widget(),
                append: inc_button = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                    add_css_class: iterate!(&model.classes),
                },
                append = &gtk::Button::new() {
                    set_label: track!(model.decrement, &format!("Last decrement at {}", model.counter)),
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
                        set_label: track! {&model.counter.to_string()},
                    },
                    attach(1, 2, 1, 1): label2 = &gtk::Label {
                        set_label: "grid test 2",
                        set_visible: counter == 0,
                    },
                    attach(2, 1, 1, 1) = &gtk::Label {
                        set_label: watch!(&format!("dbg: {:?}", label2)),
                    },
                    attach(2, 2, 1, 1): components.button2.root_widget(),
                },
                append: stack = &gtk::Stack {
                    add_child = &gtk::Label {
                        set_label: "Testing StackPage 1",
                    } -> {
                        set_title: "Test page 1",
                    },
                    add_child = &gtk::Label {
                        set_label: "Testing StackPage 2",
                    } -> test_page: gtk::StackPage {
                        set_title: "Test page 2",
                    },
                },
                append = &gtk::StackSwitcher {
                    set_stack: Some(&stack)
                },
                append: &vbox,
                append = &gtk::Button {
                    connect_clicked[sender1 = components.button1.sender()] => move |_| {
                        send!(sender1, ButtonMsg);
                    }
                },
                append = &gtk::CenterBox {
                    set_center_widget: watch!(Some(if model.counter % 2 == 0 {
                        test_label_1.widget_ref()
                    } else {
                        test_label_2.widget_ref()
                    })),
                }
            },
        }
    }

    additional_fields! {
        test_field: u8,
        test_label_1: gtk::Label,
        test_label_2: gtk::Label,
    }

    fn pre_init() {
        let mut test_field = 0;
        println!("Pre init! test_field: {}", test_field);

        relm4::view! {
            vbox = gtk::Box {
                append = &gtk::Button {
                    set_label: "Click me!",
                    connect_clicked => |_| {
                        println!("Hello world!");
                    },
                },
            }
        }

        let start_label = gtk::Label::builder()
            .label("This should appear at the top")
            .build();

        let counter = model.counter;

        let test_label_1 = gtk::Label::new(Some("test 1"));
        let test_label_2 = gtk::Label::new(Some("test 2"));
    }

    fn post_init() {
        relm4::set_global_css(b".first { color: green; } .second { border: 1px solid orange; }");
        test_field = 42;
        println!("Post init! test_field: {}", test_field);
    }

    fn pre_view() {
        self.test_page.set_title("Manually set title in view!");
        self.test_field += 1;
        println!("Manual view! test_field: {}", self.test_field);
    }
}

fn main() {
    let model = AppModel {
        counter: 0,
        classes: vec!["first", "second"],
        decrement: false,
        tracker: 0,
    };
    let app = RelmApp::new(model);

    // Test macro ordering
    relm4::view! {
         mut vec = Vec::new() {
             push: gtk::Label::new(Some("5")),
             push = gtk::Label {
                 set_label: "6",
             }
        }
    }

    app.run();
}
