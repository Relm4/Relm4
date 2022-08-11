use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::actions::{AccelsPlus, ActionablePlus, RelmAction, RelmActionGroup};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, SimpleComponent, WidgetPlus};

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
    type InitParams = u8;
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        #[root]
        main_window = gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    //set_action::<TestU8Action>: 1,
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Increment);
                    },
                },
                gtk::Button::with_label("Decrement") {
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Decrement);
                    },
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
            "Test" => TestAction,
            "Test2" => TestAction,
            "Test toggle" => TestU8Action(1_u8),
            section! {
                "Section test" => TestAction,
                "Test toggle" => TestU8Action(1_u8),
            },
            section! {
                "Test" => TestAction,
                "Test2" => TestAction,
                "Test Value" => TestU8Action(1_u8),
            }
        }
    }

    fn init(
        counter: Self::InitParams,
        root: &Self::Root,
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
        //         "Test" => TestAction,
        //         "Test2" => TestAction,
        //         "Test toggle" => TestU8Action(1_u8),
        //         section! {
        //             "Section test" => TestAction,
        //             "Test toggle" => TestU8Action(1_u8),
        //         },
        //         section! {
        //             "Test" => TestAction,
        //             "Test2" => TestAction,
        //             "Test Value" => TestU8Action(1_u8),
        //         }
        //     }
        // };

        let model = Self { counter };
        let widgets = view_output!();

        let group = RelmActionGroup::<WindowActionGroup>::new();

        let action: RelmAction<TestAction> = {
            RelmAction::new_stateless(move |_| {
                println!("Statelesss action!");
                sender.input(AppMsg::Increment);
            })
        };

        let action2: RelmAction<TestU8Action> =
            RelmAction::new_stateful_with_target_value(&0, |_, state, _value| {
                *state ^= 1;
                dbg!(state);
            });

        group.add_action(action);
        group.add_action(action2);

        let actions = group.into_action_group();
        widgets
            .main_window
            .insert_action_group("win", Some(&actions));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(TestAction, WindowActionGroup, "test");
relm4::new_stateful_action!(TestU8Action, WindowActionGroup, "test2", u8, u8);

fn main() {
    let app = RelmApp::new("relm4.test.menu");
    app.app
        .set_accelerators_for_action::<TestAction>(&["<primary>W"]);

    app.run::<AppModel>(0);
}
