use async_trait::async_trait;
use gtk::prelude::*;
use relm4::*;
use relm4::factory::{AsyncFactoryVecDeque, AsyncFactoryComponent};
use relm4::prelude::*;

struct Window {
    counter: u8,
}

#[derive(Debug)]
enum WindowMsg {
    Increment,
    Decrement,
}

#[relm4::factory(async)]
impl AsyncFactoryComponent for Window {
    type Init = u8;
    type Input = WindowMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::Application;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_visible: true,
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => WindowMsg::Increment,
                },

                gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => WindowMsg::Decrement,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Counter: {}", &self.counter),
                    set_margin_all: 5,
                }
            }
        }
    }

    // Initialize the component.
    async fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: AsyncFactorySender<Self>) -> Self {
        Self { counter: value }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncFactorySender<Self>) {
        match msg {
            WindowMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            WindowMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

struct App {
    windows: AsyncFactoryVecDeque<Window>,
}

#[derive(Debug)]
enum AppMsg {
    Activate(u8),
}

#[relm4::component(async)]
impl SimpleAsyncComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        main_adw_application() -> adw::Application {
            connect_activate => AppMsg::Activate(init)
        }
    }

    // Initialize the component.
    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = App {
            windows: AsyncFactoryVecDeque::builder()
                .launch(root.clone())
                .detach()
         };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        let mut windows_guard = self.windows.guard();

        match msg {
            AppMsg::Activate(init) => {
                windows_guard.push_back(init);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.application");
    app.run_application_async::<App>(0);
}
