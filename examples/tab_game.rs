use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    Component, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SharedState,
    factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque},
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
impl FactoryComponent for GamePage {
    type Init = u8;
    type Input = CounterMsg;
    type Output = CounterOutput;
    type CommandOutput = ();
    type ParentWidget = adw::TabView;

    view! {
        #[root]
        root = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,

            gtk::CenterBox {
                set_hexpand: true,
                set_vexpand: true,

                #[wrap(Some)]
                set_center_widget = match *state {
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
                                connect_clicked[sender, index] => move |_| {
                                    sender.output(CounterOutput::StartGame(index.clone())).unwrap()
                                }
                            },
                        }
                    }
                    GameState::Guessing => {
                        gtk::Button {
                            set_label: "This was my tab!",
                            set_valign: gtk::Align::Center,

                            connect_clicked[sender, index] => move |_| {
                                sender.output(CounterOutput::SelectedGuess(index.clone())).unwrap()
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
                                    *GAME_STATE.write() = GameState::Start;
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
            set_title: &match *state {
                GameState::Running | GameState::Guessing => {
                    "???".to_string()
                }
                _ => format!("Tab {}", self.id),
            },
            #[watch]
            set_loading: matches!(*state, GameState::Running),
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        GAME_STATE.subscribe(sender.input_sender(), |_| CounterMsg::Update);
        Self { id: value }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        returned_widget: &adw::TabPage,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let state = GAME_STATE.read();
        let widgets = view_output!();
        widgets
    }

    fn pre_view() {
        let state = GAME_STATE.read();
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            CounterMsg::Update => (),
        }
    }
}

struct App {
    counters: FactoryVecDeque<GamePage>,
    start_index: Option<DynamicIndex>,
}

#[derive(Debug)]
enum AppMsg {
    SelectedGuess(DynamicIndex),
    StartGame(DynamicIndex),
    StopGame,
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = bool;

    view! {
        adw::Window {
            set_title: Some("Tab game!"),
            set_default_size: (400, 200),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                adw::HeaderBar {},

                adw::TabBar {
                    set_view: Some(tab_view),
                    set_autohide: false,
                },

                #[local_ref]
                tab_view -> adw::TabView {
                    connect_close_page => |_, _| {
                        gtk::glib::signal::Propagation::Stop
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let counters = FactoryVecDeque::builder()
            .launch(adw::TabView::default())
            .forward(sender.input_sender(), |output| match output {
                CounterOutput::StartGame(index) => AppMsg::StartGame(index),
                CounterOutput::SelectedGuess(guess) => AppMsg::SelectedGuess(guess),
            });

        let mut model = App {
            counters,
            start_index: None,
        };

        let tab_view = model.counters.widget();
        let widgets = view_output!();

        let mut counters_guard = model.counters.guard();
        for i in 0..3 {
            counters_guard.push_back(i);
        }

        // Explicitly drop the guard,
        // so that 'model' is no longer borrowed
        // and can be moved inside ComponentParts
        counters_guard.drop();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppMsg::StartGame(index) => {
                self.start_index = Some(index);
                sender.command(|sender, _| async move {
                    for i in (1..4).rev() {
                        *GAME_STATE.write() = GameState::Countdown(i);
                        relm4::tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                    *GAME_STATE.write() = GameState::Running;
                    for _ in 0..20 {
                        relm4::tokio::time::sleep(Duration::from_millis(500)).await;
                        sender.send(false).unwrap();
                    }
                    relm4::tokio::time::sleep(Duration::from_millis(1000)).await;
                    sender.send(true).unwrap();
                });
            }
            AppMsg::StopGame => {
                *GAME_STATE.write() = GameState::Guessing;
            }
            AppMsg::SelectedGuess(index) => {
                *GAME_STATE.write() = GameState::End(index == self.start_index.take().unwrap());
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if msg {
            sender.input(AppMsg::StopGame);
        } else {
            let mut counters_guard = self.counters.guard();
            match rand::random::<u8>() % 3 {
                0 => {
                    counters_guard.swap(1, 2);
                }
                1 => {
                    counters_guard.swap(0, 1);
                }
                _ => {
                    let widget = counters_guard.widget();
                    if !widget.select_next_page() {
                        widget.select_previous_page();
                    }
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.tab_game");
    app.run::<App>(());
}
