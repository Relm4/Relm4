use gtk::prelude::*;
use relm4::actions::*;
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
        main_window = gtk::ApplicationWindow {
            set_title: Some("Action example"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    ActionablePlus::set_action::<TestU8Action>: 1,
                    connect_clicked => AppMsg::Increment,
                },

                gtk::Button::with_label("Decrement") {
                    connect_clicked => AppMsg::Decrement,
                },

                gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },

                gtk::MenuButton {
                    set_menu_model: Some(&menu_model),
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let menu_model = gtk::gio::Menu::new();
        menu_model.append(Some("Stateless"), Some(&TestAction::action_name()));

        let model = Self { counter: 0 };

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<TestAction>(&["<primary>W"]);

        let group = RelmActionGroup::<WindowActionGroup>::new();

        let action: RelmAction<TestAction> = RelmAction::new_stateless(move |_| {
            println!("Statelesss action!");
            sender.input(AppMsg::Increment);
        });

        let action2: RelmAction<TestU8Action> =
            RelmAction::new_stateful_with_target_value(&0, |_, state, value| {
                println!("Stateful action -> state: {}, value: {}", state, value);
                *state += value;
            });

        group.add_action(action);
        group.add_action(action2);

        let actions = group.into_action_group();
        widgets
            .main_window
            .insert_action_group(WindowActionGroup::NAME, Some(&actions));

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

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(TestAction, WindowActionGroup, "test");
relm4::new_stateful_action!(TestU8Action, WindowActionGroup, "test2", u8, u8);

fn main() {
    let app = RelmApp::new("relm4.example.actions");
    app.run::<AppModel>(());
}
