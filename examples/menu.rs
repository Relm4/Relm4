use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::actions::{AccelsPlus, ActionablePlus, RelmAction, RelmActionGroup};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

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
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        #[root]
        main_window = gtk::ApplicationWindow {
            set_title: Some("Menu example"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => Msg::Increment,
                    ActionablePlus::set_action::<ExampleU8Action>: 1,
                },
                gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => Msg::Decrement,
                },
                gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },
                gtk::MenuButton {
                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {
                        add_child: (&popover_child, "my_widget"),
                    }
                }
            },
        },
        popover_child = gtk::Spinner {
            set_spinning: true,
        }
    }

    menu! {
        main_menu: {
            custom: "my_widget",
            "Example" => ExampleAction,
            "Example2" => ExampleAction,
            "Example toggle" => ExampleU8Action(1_u8),
            section! {
                "Section example" => ExampleAction,
                "Example toggle" => ExampleU8Action(1_u8),
            },
            section! {
                "Example" => ExampleAction,
                "Example2" => ExampleAction,
                "Example Value" => ExampleU8Action(1_u8),
            },
            "submenu1" {
                "Example" => ExampleAction,
                "Example2" => ExampleAction,
                "Example toggle" => ExampleU8Action(1_u8),
                "submenu2" {
                    "Example" => ExampleAction,
                    "Example2" => ExampleAction,
                    "Example toggle" => ExampleU8Action(1_u8),
                    "submenu3" {
                        "Example" => ExampleAction,
                        "Example2" => ExampleAction,
                        "Example toggle" => ExampleU8Action(1_u8),
                    }
                }
            }
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // ============================================================
        //
        // You can also use menu! outside of the widget macro.
        // This is the manual equivalent to the the menu! macro above.
        //
        // ============================================================
        //
        // relm4::menu! {
        //     main_menu: {
        //         custom: "my_widget",
        //         "Example" => ExampleAction,
        //         "Example2" => ExampleAction,
        //         "Example toggle" => ExampleU8Action(1_u8),
        //         section! {
        //             "Section example" => ExampleAction,
        //             "Example toggle" => ExampleU8Action(1_u8),
        //         },
        //         section! {
        //             "Example" => ExampleAction,
        //             "Example2" => ExampleAction,
        //             "Example Value" => ExampleU8Action(1_u8),
        //         }
        //     }
        // };

        let model = Self { counter };
        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<ExampleAction>(&["<primary>W"]);

        let action: RelmAction<ExampleAction> = {
            RelmAction::new_stateless(move |_| {
                println!("Statelesss action!");
                sender.input(Msg::Increment);
            })
        };

        let action2: RelmAction<ExampleU8Action> =
            RelmAction::new_stateful_with_target_value(&0, |_, state, _value| {
                *state ^= 1;
                dbg!(state);
            });

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(action);
        group.add_action(action2);
        group.register_for_widget(&widgets.main_window);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(ExampleAction, WindowActionGroup, "example");
relm4::new_stateful_action!(ExampleU8Action, WindowActionGroup, "example2", u8, u8);

fn main() {
    let app = RelmApp::new("relm4.example.menu");
    app.run::<App>(0);
}
