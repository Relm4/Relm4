use std::convert::identity;
use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    gtk, Component, ComponentParts, ComponentSender, RelmApp, SimpleComponent, WidgetPlus, Worker,
    WorkerController,
};

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
    type InitParams = ();
    type Input = AsyncHandlerMsg;
    type Output = AppMsg;

    fn init(_params: Self::InitParams, _sender: ComponentSender<Self>) -> Self {
        Self
    }

    // This is blocking on purpose.
    // Only one message can be processed at the time.
    // If you don't want to block during processing, look for commands.
    // You'll find a good reference in the "non_blocking_async" example.
    fn update(&mut self, msg: AsyncHandlerMsg, sender: ComponentSender<Self>) {
        let output = sender.output.clone();

        std::thread::sleep(Duration::from_secs(1));

        match msg {
            AsyncHandlerMsg::DelayedIncrement => output.send(AppMsg::Increment),
            AsyncHandlerMsg::DelayedDecrement => output.send(AppMsg::Decrement),
        }
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
            set_title: Some("Worker Counter"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked[sender = model.worker.sender().clone()] => move |_| {
                        sender.send(AsyncHandlerMsg::DelayedIncrement);
                    },
                },
                gtk::Button::with_label("Decrement") {
                    connect_clicked[sender = model.worker.sender().clone()] => move |_| {
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

    fn init(_: (), root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = AppModel {
            counter: 0,
            worker: AsyncHandler::builder()
                .detach_worker(())
                .forward(&sender.input, identity),
        };

        let widgets = view_output!();

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

fn main() {
    let app = RelmApp::new("relm4.test.worker");
    app.run::<AppModel>(());
}
