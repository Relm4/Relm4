use gtk::prelude::*;
use relm4::actions::*;
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
        main_window = gtk::ApplicationWindow {
            set_title: Some("Action example"),
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

                gtk::Button::with_label("Decrement") {
                    connect_clicked => Msg::Decrement,
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
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let menu_model = gtk::gio::Menu::new();
        menu_model.append(Some("Stateless"), Some(&ExampleAction::action_name()));

        let model = Self { counter: 0 };

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<ExampleAction>(&["<primary>W"]);

        let action: RelmAction<ExampleAction> = RelmAction::new_stateless(move |_| {
            println!("Statelesss action!");
            sender.input(Msg::Increment);
        });

        let action2: RelmAction<ExampleU8Action> =
            RelmAction::new_stateful_with_target_value(&0, |_, state, value| {
                println!("Stateful action -> state: {state}, value: {value}");
                *state += value;
            });

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(action);
        group.add_action(action2);
        group.register_for_widget(&widgets.main_window);

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

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(ExampleAction, WindowActionGroup, "example");
relm4::new_stateful_action!(ExampleU8Action, WindowActionGroup, "example2", u8, u8);

fn main() {
    let app = RelmApp::new("relm4.example.actions");
    app.run::<App>(());
}
