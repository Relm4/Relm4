use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    adw,
    factory::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    gtk, Component, ComponentParts, ComponentSender, RelmApp, Sender, SharedState, WidgetPlus,
};

enum GameState {
    Start,
    Countdown(u8),
    Running,
    Guessing,
    End(bool),
}

impl Default for GameState {
    fn default() -> Self {
        Self::Start
    }
}

static GAME_STATE: SharedState<GameState> = SharedState::new();

#[derive(Debug)]
struct GamePage {
    id: u8,
}

#[derive(Debug)]
enum CounterMsg {
    Update,
}

#[derive(Debug)]
enum CounterOutput {
    StartGame(DynamicIndex),
    SelectedGuess(DynamicIndex),
}

#[relm4::factory]
impl FactoryComponent<adw::TabView, AppMsg> for GamePage {
    type Widgets = CounterWidgets;

    type InitParams = u8;

    type Input = CounterMsg;
    type Output = CounterOutput;

    type Command = ();
    type CommandOutput = ();

    fn output_to_parent_msg(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            CounterOutput::StartGame(index) => AppMsg::StartGame(index),
            CounterOutput::SelectedGuess(guess) => AppMsg::SelectedGuess(guess),
        })
    }

    view! {
        #[root]
        root = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,

            gtk::CenterBox {
                set_hexpand: true,
                set_vexpand: true,

                #[wrap(Some)]
                set_center_widget = match **state {
                    GameState::Countdown(value) => {
                        gtk::Label {
                            set_valign: gtk::Align::Center,

                            #[watch]
                            set_label: &value.to_string(),
                        }
                    }
                    GameState::Running => {
                        gtk::Label {
                            set_valign: gtk::Align::Center,
                            set_label: "???",
                        }
                    }
                    GameState::Start => {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_margin_all: 10,
                            set_spacing: 10,

                            gtk::Label {
                                set_label: "Can you still find this tab after is was shuffled?",
                            },
                            gtk::Button {
                                set_label: "Start!",
                                connect_clicked[output, index] => move |_| {
                                    output.send(CounterOutput::StartGame(index.clone()));
                                }
                            },
                        }
                    }
                    GameState::Guessing => {
                        gtk::Button {
                            set_label: "This was my tab!",
                            set_valign: gtk::Align::Center,

                            connect_clicked[output, index] => move |_| {
                                output.send(CounterOutput::SelectedGuess(index.clone()));
                            }
                        }
                    }
                    GameState::End(won) => {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_margin_all: 10,
                            set_spacing: 10,

                            gtk::Label {
                                #[watch]
                                set_label: if won {
                                        "That's correct, you win!"
                                    } else {
                                        "You lose, this wasn't your tab..."
                                    },
                            },
                            gtk::Button {
                                set_label: "Start again",
                                connect_clicked => move |_| {
                                    **GAME_STATE.get_mut() = GameState::Start;
                                }
                            },
                        }

                    }
                }
            }
        },
        #[local_ref]
        returned_widget -> adw::TabPage {
            #[watch]
            set_title: &match **state {
                GameState::Running | GameState::Guessing => {
                    "???".to_string()
                }
                _ => format!("Tab {}", self.id),
            },
            #[watch]
            set_loading: matches!(**state, GameState::Running),
        }
    }

    fn init_model(
        value: Self::InitParams,
        _index: &DynamicIndex,
        input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        GAME_STATE.subscribe(input, |_| CounterMsg::Update);
        Self { id: value }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: &Self::Root,
        returned_widget: &adw::TabPage,
        _input: &Sender<Self::Input>,
        output: &Sender<Self::Output>,
    ) -> Self::Widgets {
        let state = GAME_STATE.get();
        let widgets = view_output!();
        widgets
    }

    fn pre_view() {
        let state = GAME_STATE.get();
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _input: &Sender<Self::Input>,
        _ouput: &Sender<Self::Output>,
    ) -> Option<Self::Command> {
        match msg {
            CounterMsg::Update => (),
        }
        None
    }
}

struct AppModel {
    counters: FactoryVecDeque<adw::TabView, GamePage, AppMsg>,
    start_index: Option<DynamicIndex>,
}

#[derive(Debug)]
enum AppMsg {
    SelectedGuess(DynamicIndex),
    StartGame(DynamicIndex),
    StopGame,
}

#[relm4::component]
impl Component for AppModel {
    // AppWidgets is generated by the macro
    type Widgets = AppWidgets;

    type InitParams = ();

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = bool;

    view! {
        adw::Window {
            set_title: Some("Tab game!"),
            set_default_width: 400,
            set_default_height: 200,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                adw::HeaderBar {},

                adw::TabBar {
                    set_view: Some(&tabs),
                    set_autohide: false,
                },

                append: tabs = &adw::TabView {
                    connect_close_page => |_, _| {
                        true
                    }
                }
            }
        }
    }

    // Initialize the UI.
    fn init(
        _init_params: (),
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Insert the macro codegen here
        let widgets = view_output!();

        let mut model = AppModel {
            counters: FactoryVecDeque::new(widgets.tabs.clone(), &sender.input),
            start_index: None,
        };
        for i in 0..3 {
            model.counters.push_back(i);
        }
        model.counters.render_changes();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        self.counters.apply_external_updates();

        match msg {
            AppMsg::StartGame(index) => {
                self.start_index = Some(index);
                sender.command(|sender, _| async move {
                    for i in (1..4).rev() {
                        **GAME_STATE.get_mut() = GameState::Countdown(i);
                        relm4::tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                    **GAME_STATE.get_mut() = GameState::Running;
                    for _ in 0..20 {
                        relm4::tokio::time::sleep(Duration::from_millis(500)).await;
                        sender.send(false);
                    }
                    relm4::tokio::time::sleep(Duration::from_millis(1000)).await;
                    sender.send(true);
                });
            }
            AppMsg::StopGame => {
                **GAME_STATE.get_mut() = GameState::Guessing;
            }
            AppMsg::SelectedGuess(index) => {
                **GAME_STATE.get_mut() = GameState::End(index == self.start_index.take().unwrap());
            }
        }
        self.counters.render_changes();
    }

    fn update_cmd(&mut self, msg: Self::CommandOutput, sender: &ComponentSender<Self>) {
        if msg {
            sender.input(AppMsg::StopGame);
        } else {
            self.counters.apply_external_updates();
            match rand::random::<u8>() % 3 {
                0 => {
                    self.counters.swap(1, 2);
                }
                1 => {
                    self.counters.swap(0, 1);
                }
                _ => {
                    let widget = self.counters.widget();
                    if !widget.select_next_page() {
                        widget.select_previous_page();
                    }
                }
            }
            self.counters.render_changes();
        }
    }
}

fn main() {
    let app: RelmApp<AppModel> = RelmApp::new("relm4.test.tabGame");
    app.run(());
}
