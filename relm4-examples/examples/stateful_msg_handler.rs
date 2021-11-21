use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    send, AppUpdate, MessageHandler, Model, RelmApp, RelmMsgHandler, Sender, WidgetPlus, Widgets,
};

use std::sync::Mutex;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};

struct AppModel {
    counter: u8,
}

#[derive(Debug)]
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

struct AsyncHandler {
    _rt: Runtime,
    sender: TokioSender<AsyncHandlerMsg>,
}

#[derive(Debug)]
enum AsyncHandlerMsg {
    DelayedIncrement,
    DelayedDecrement,
}

impl MessageHandler<AppModel> for AsyncHandler {
    type Msg = AsyncHandlerMsg;
    type Sender = TokioSender<AsyncHandlerMsg>;

    fn init(_parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        let (sender, mut rx) = channel::<AsyncHandlerMsg>(10);

        let rt = Builder::new_multi_thread()
            .worker_threads(8)
            .enable_time()
            .build()
            .unwrap();

        let mut state = Mutex::new(0u8);

        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(state) = state.get_mut() {
                    *state += 1;
                    println!("Received message No. {}", state);
                }

                let parent_sender = parent_sender.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    match msg {
                        AsyncHandlerMsg::DelayedIncrement => {
                            send!(parent_sender, AppMsg::Increment);
                        }
                        AsyncHandlerMsg::DelayedDecrement => {
                            send!(parent_sender, AppMsg::Decrement);
                        }
                    }
                });
            }
        });

        AsyncHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}

#[derive(relm4_macros::Components)]
struct AppComponents {
    async_handler: RelmMsgHandler<AsyncHandler, AppModel>,
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append = &gtk::Button {
                    set_label: "Increment",
                    connect_clicked[sender = components.async_handler.sender()] => move |_| {
                        sender.blocking_send(AsyncHandlerMsg::DelayedIncrement).expect("Receiver dropped");
                    },
                },
                append = &gtk::Button::with_label("Decrement") {
                    connect_clicked[sender = components.async_handler.sender()] => move |_| {
                        sender.blocking_send(AsyncHandlerMsg::DelayedDecrement).expect("Receiver dropped");
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                }
            },
        }
    }
}

fn main() {
    let model = AppModel { counter: 0 };
    let app = RelmApp::new(model);
    app.run();
}
