use std::convert::identity;
use std::time::Duration;

use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{
    gtk, Component, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent,
    Worker, WorkerController,
};

struct AsyncHandler;

#[derive(Debug)]
enum AsyncHandlerMsg {
    DelayedIncrement,
    DelayedDecrement,
}

struct App {
    counter: u8,
    worker: WorkerController<AsyncHandler>,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

impl Worker for AsyncHandler {
    type Init = ();
    type Input = AsyncHandlerMsg;
    type Output = AppMsg;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self
    }

    // This is blocking on purpose.
    // Only one message can be processed at the time.
    // If you don't want to block during processing, look for commands.
    // You'll find a good reference in the "non_blocking_async" example.
    fn update(&mut self, msg: AsyncHandlerMsg, sender: ComponentSender<Self>) {
        std::thread::sleep(Duration::from_secs(1));

        match msg {
            AsyncHandlerMsg::DelayedIncrement => sender.output(AppMsg::Increment).unwrap(),
            AsyncHandlerMsg::DelayedDecrement => sender.output(AppMsg::Decrement).unwrap(),
        }
    }
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Worker Counter"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked[sender = model.worker.sender().clone()] => move |_| {
                        sender.send(AsyncHandlerMsg::DelayedIncrement).unwrap();
                    },
                },
                gtk::Button::with_label("Decrement") {
                    connect_clicked[sender = model.worker.sender().clone()] => move |_| {
                        sender.send(AsyncHandlerMsg::DelayedDecrement).unwrap();
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

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            counter: 0,
            worker: AsyncHandler::builder()
                .detach_worker(())
                .forward(sender.input_sender(), identity),
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
    let app = RelmApp::new("relm4.example.worker");
    app.run::<App>(());
}
