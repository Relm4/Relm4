use std::thread;

use gtk::prelude::*;
use relm4::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

struct Window {
    counter: u8,
}

#[derive(Debug)]
enum WindowMsg {
    Increment,
    Decrement,
}

#[relm4::factory]
impl FactoryComponent for Window {
    type Init = u8;
    type Input = WindowMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::Application;

    view! {
        #[root]
        gtk::Window {
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
    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { counter: value }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
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
    hold_guard: Option<ApplicationHoldGuard>,
    windows: FactoryVecDeque<Window>,
}

#[derive(Debug)]
enum AppMsg {
    Idle,
    Activate(u8),
}

impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();
    type Root = adw::Application;
    type Widgets = ();

    fn init_root() -> Self::Root {
        main_adw_application()
    }

    // Initialize the component.
    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let model = App {
            hold_guard: Some(root.hold()),
            windows: FactoryVecDeque::builder()
                .launch(root.clone())
                .detach()
         };

        let csender = sender.clone();
        root.connect_activate(move |_app| {
            csender.input(AppMsg::Activate(init))
        });


        sender.input(AppMsg::Idle);
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        let mut windows_guard = self.windows.guard();

        match msg {
            AppMsg::Idle => (),
            AppMsg::Activate(init) => {
                windows_guard.push_back(init);
                self.hold_guard.take();
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.application");
    app.run_with_application::<App>(0);
}
