use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    actions::{
        AccelsPlus, ActionGroupName, ActionName, ActionablePlus, RelmAction, RelmActionGroup,
    },
    send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets,
};

#[derive(Default)]
struct AppModel {
    counter: u8,
}

enum AppMsg {
    Increment,
    Decrement,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    set_action<TestU8Action>: 1,
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Increment);
                    },
                },
                append = &gtk::Button::with_label("Decrement") {
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::Decrement);
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                },
                append = &gtk::MenuButton {
                    set_menu_model: Some(&menu_model),
                }
            },
        }
    }

    fn pre_init() {
        let menu_model = gtk::gio::Menu::new();
        menu_model.append(Some("Stateless"), Some(&TestAction::action_name()));
    }

    fn post_init() {
        let app = relm4::gtk_application();
        app.set_accelerators_for_action::<TestAction>(&["<primary>W"]);

        let group = RelmActionGroup::<WindowActionGroup>::new();

        let action: RelmAction<TestAction> = RelmAction::new_stateless(move |_| {
            println!("Statelesss action!");
            send!(sender, AppMsg::Increment);
        });

        let action2: RelmAction<TestU8Action> =
            RelmAction::new_stateful_with_target_value(&0, |_, state, value| {
                println!("Stateful action -> state: {}, value: {}", state, value);
                *state += value;
            });

        group.add_action(action);
        group.add_action(action2);

        let actions = group.into_action_group();
        main_window.insert_action_group("win", Some(&actions));
    }
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(TestAction, WindowActionGroup, "test");
relm4::new_stateful_action!(TestU8Action, WindowActionGroup, "test2", u8, u8);

fn main() {
    let model = AppModel::default();

    let app = RelmApp::new(model);
    app.run();
}
