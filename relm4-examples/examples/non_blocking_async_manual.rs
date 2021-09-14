use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

use tokio::runtime::Builder;
use tokio::sync::mpsc::{channel, Sender as TokioSender};

struct AppModel {
    counter: u8,
    async_handler: TokioSender<(AsyncHandlerMsg, Sender<AppMsg>)>,
}

#[derive(Debug)]
enum AsyncHandlerMsg {
    IncrementRequest,
    DecrementRequest,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
    type Settings = ();
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
                    connect_clicked(sender, async_sender) => move |_| {
                        async_sender.blocking_send((AsyncHandlerMsg::IncrementRequest, sender.clone())).unwrap();
                    },
                },
                append = &gtk::Button::with_label("Decrement") {
                    connect_clicked(sender, async_sender) => move |_| {
                        async_sender.blocking_send((AsyncHandlerMsg::DecrementRequest, sender.clone())).unwrap();
                    },
                },
                append = &gtk::Label {
                    set_margin_all: 5,
                    set_label: watch! { &format!("Counter: {}", model.counter) },
                }
            },
        }
    }

    fn pre_init() {
        let async_sender = &model.async_handler;
    }
}

fn main() {
    let (rx, mut tx) = channel::<(AsyncHandlerMsg, Sender<AppMsg>)>(10);

    let rt = Builder::new_multi_thread()
        .worker_threads(8)
        .enable_time()
        .build()
        .unwrap();

    rt.spawn(async move {
        while let Some((msg, sender)) = tx.recv().await {
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                match msg {
                    AsyncHandlerMsg::IncrementRequest => {
                        send!(sender, AppMsg::Increment);
                    }
                    AsyncHandlerMsg::DecrementRequest => {
                        send!(sender, AppMsg::Decrement);
                    }
                }
            });
        }
    });

    let model = AppModel {
        counter: 0,
        async_handler: rx,
    };
    let app = RelmApp::new(model, &());
    app.run();
}
