use gtk::prelude::*;
use relm4::{
    component, gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};

#[derive(Default)]
struct StackApp {
    stack: gtk::Stack,
}

#[derive(Debug)]
enum StackAppMsg {
    SwitchPage(u8),
}

#[component]
impl SimpleComponent for StackApp {
    type Init = ();
    type Input = StackAppMsg;
    type Output = ();
    type Widgets = StackAppWidgets;

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_default_size: (300, 200),

            #[wrap(Some)]
            set_titlebar = &gtk::HeaderBar {
                
                #[wrap(Some)]
                set_title_widget = &gtk::StackSwitcher {
                    set_stack: Some(&stack),
                    set_halign: gtk::Align::Center,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 6,
                set_margin_all: 12,

                #[name = "stack"]
                gtk::Stack {
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                    set_transition_duration: 200,

                    add_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_margin_all: 20,
                        
                        gtk::Label {
                            set_label: "Welcome to page 1",
                            set_margin_bottom: 20,
                        },

                        gtk::Button {
                            set_label: "Go to page 2",
                            connect_clicked => StackAppMsg::SwitchPage(2),
                        },
                    } -> {
                        set_name: "1",
                        set_title: "page 1",
                    },

                    add_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_margin_all: 20,
                        
                        gtk::Label {
                            set_label: "This is page 2!",
                            set_margin_bottom: 20,
                        },

                        gtk::Button {
                            set_label: "Go to page 1",
                            connect_clicked => StackAppMsg::SwitchPage(1),
                        },
                    } -> {
                        set_name: "2",
                        set_title: "page 2",
                    },

                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let mut model = StackApp::default();
        model.stack = widgets.stack.clone();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            StackAppMsg::SwitchPage(page) => {
                println!("Go to page {}", page);
                self.stack.set_visible_child_name(&page.to_string());
            }
        }
    }
}

fn main() {
    let app = relm4::RelmApp::new("relm4.example.stack");
    app.run::<StackApp>(());
}