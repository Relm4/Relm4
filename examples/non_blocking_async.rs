use std::convert::identity;
use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    gtk, worker::WorkerFuture, ComponentParts, ComponentSender, RelmApp, Sender, SimpleComponent,
    WidgetPlus, Worker, WorkerController,
};
use tokio::time::sleep;

struct AsyncHandler;

#[derive(Debug)]
enum AsyncHandlerMsg {
    DelayedIncrement,
    DelayedDecrement,
}

struct AppModel {
    counter: u8,
    worker: WorkerController<AsyncHandler>,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

impl Worker for AsyncHandler {
    type InputParams = ();
    type Input = AsyncHandlerMsg;
    type Output = AppMsg;

    fn init_inner(_: (), _: &mut Sender<AsyncHandlerMsg>, _: &mut Sender<AppMsg>) -> Self {
        AsyncHandler
    }

    fn update(
        &mut self,
        msg: AsyncHandlerMsg,
        _: &mut Sender<AsyncHandlerMsg>,
        output: &mut Sender<AppMsg>,
    ) -> WorkerFuture {
        let output = output.clone();

        Box::pin(async move {
            sleep(Duration::from_secs(1)).await;

            match msg {
                AsyncHandlerMsg::DelayedIncrement => output.send(AppMsg::Increment),
                AsyncHandlerMsg::DelayedDecrement => output.send(AppMsg::Decrement),
            }
        })
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type InitParams = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Async Counter"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked[sender = model.worker.sender.clone()] => move |_| {
                        sender.send(AsyncHandlerMsg::DelayedIncrement);
                    },
                },
                gtk::Button::with_label("Decrement") {
                    connect_clicked[sender = model.worker.sender.clone()] => move |_| {
                        sender.send(AsyncHandlerMsg::DelayedDecrement);
                    },
                },
                gtk::Label {
                    set_margin_all: 5,
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                },
            },
        }
    }

    fn init(_: (), root: &Self::Root, sender: &ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AppModel {
            counter: 0,
            worker: AsyncHandler::init(()).forward(&sender.input, identity),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: &ComponentSender<Self>) {
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

fn main() {
    let app: RelmApp<AppModel> = RelmApp::new("relm4.test.non_blocking_async");
    app.run(());
}
