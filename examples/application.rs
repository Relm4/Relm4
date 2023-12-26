use std::collections::VecDeque;

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
    type ParentWidget = gtk::Application;

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
    queued_windows: VecDeque<u8>,
    windows: FactoryVecDeque<Window>,
}

#[derive(Debug)]
enum AppMsg {
    Activate(u8),
}

impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();
    type Root = adw::Application;
    type Widgets = ();

    fn init_root() -> Self::Root {
        let app = main_adw_application();
        app
    }

    // Initialize the component.
    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = App {
            queued_windows: VecDeque::new(),
            windows: FactoryVecDeque::builder()
                .launch_default()
                .detach()
         };

        let csender = sender.clone();
        root.connect_activate(move |_app| {
            println!("activate");
            csender.input(AppMsg::Activate(init))
        });

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        let mut windows_guard = self.windows.guard();

        match msg {
            AppMsg::Activate(init) => {
                println!("Add window");
                self.queued_windows.push_back(init);
                           windows_guard.push_back(init);

            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.application");
    app.run_with_application::<App>(0);
}
