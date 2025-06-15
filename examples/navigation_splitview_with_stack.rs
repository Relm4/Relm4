// Required feature: libadwaita,gnome_45

use relm4::RelmApp;

fn main() {
    let app = RelmApp::new("relm4.example.navigation_stack");
    app.run::<app::App>((0, false));
}

// The main app
mod app {
    use crate::{counter::CounterModel, toggler::TogglerModel};
    use adw::prelude::{AdwApplicationWindowExt, IsA, NavigationPageExt, ToValue};
    use gtk::glib;
    use relm4::{
        adw, view, Component, ComponentController, ComponentParts, ComponentSender, Controller,
        SimpleComponent,
    };
    use std::convert::identity;

    pub struct App {
        _counter: Controller<CounterModel>, // must hold on to this, would otherwise crash
        _toggler: Controller<TogglerModel>, // must hold on to this, would otherwise crash
    }

    #[derive(Debug)]
    pub enum Msg {}

    #[relm4::component(pub)]
    impl SimpleComponent for App {
        type Init = (u8, bool);
        type Input = Msg;
        type Output = ();

        view! {
            #[root]
            adw::ApplicationWindow {
                #[name(split_view)]
                adw::NavigationSplitView {
                    #[wrap(Some)]
                    set_sidebar = &adw::NavigationPage {
                        set_title: "Sidebar",

                        #[wrap(Some)]
                        set_child = &adw::ToolbarView {
                            add_top_bar = &adw::HeaderBar {},

                            #[wrap(Some)]
                            set_content = &gtk::StackSidebar {
                                set_stack: &stack,
                            },
                        },
                    },

                    #[wrap(Some)]
                    set_content = &adw::NavigationPage {
                        set_title: "Content",

                        #[wrap(Some)]
                        set_child = &adw::ToolbarView {
                            add_top_bar = &adw::HeaderBar {},
                            set_content: Some(&stack),
                        }
                    },
                },

                add_breakpoint = bp_with_setters(
                    adw::Breakpoint::new(
                        adw::BreakpointCondition::new_length(
                            adw::BreakpointConditionLengthType::MaxWidth,
                            400.0,
                            adw::LengthUnit::Sp,
                        )
                    ),
                    &[(&split_view, "collapsed", true)]
                ),
            }
        }

        additional_fields! {
            stack: gtk::Stack,
        }

        fn init(
            init: Self::Init,
            root: Self::Root,
            sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> {
            let counter = CounterModel::builder()
                .launch(init.0)
                .forward(sender.input_sender(), identity);
            let toggler = TogglerModel::builder()
                .launch(init.1)
                .forward(sender.input_sender(), identity);

            view! {
                stack = &gtk::Stack {
                    add_titled: (counter.widget(), None, "Counter"),
                    add_titled: (toggler.widget(), None, "Toggle"),
                    set_vhomogeneous: false,
                }
            }

            let model = App {
                _counter: counter,
                _toggler: toggler,
            };

            let widgets = view_output!();

            widgets.stack.connect_visible_child_notify({
                let split_view = widgets.split_view.clone();
                move |_| {
                    split_view.set_show_content(true);
                }
            });

            ComponentParts { model, widgets }
        }

        fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
            match msg {}
        }
    }

    fn bp_with_setters(
        bp: adw::Breakpoint,
        additions: &[(&impl IsA<glib::Object>, &str, impl ToValue)],
    ) -> adw::Breakpoint {
        bp.add_setters(additions);
        bp
    }
}

// The Counter page
mod counter {
    use crate::app::Msg;
    use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
    use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

    pub struct CounterModel {
        counter: u8,
    }

    #[derive(Debug)]
    pub enum CounterMsg {
        Increment,
        Decrement,
    }

    #[relm4::component(pub)]
    impl SimpleComponent for CounterModel {
        type Init = u8;
        type Input = CounterMsg;
        type Output = Msg;

        view! {
            #[root]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => CounterMsg::Increment
                },

                gtk::Button::with_label("Decrement") {
                    connect_clicked => CounterMsg::Decrement
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                    set_margin_all: 5,
                }
            }
        }

        fn init(
            init: Self::Init,
            root: Self::Root,
            sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> {
            let model = CounterModel { counter: init };

            let widgets = view_output!();

            ComponentParts { model, widgets }
        }

        fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
            match msg {
                CounterMsg::Increment => {
                    self.counter = self.counter.wrapping_add(1);
                }
                CounterMsg::Decrement => {
                    self.counter = self.counter.wrapping_sub(1);
                }
            }
        }
    }
}

// The Toggler page
mod toggler {
    use crate::app::Msg;
    use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
    use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

    pub struct TogglerModel {
        toggle: bool,
    }

    #[derive(Debug)]
    pub enum ToggleMsg {
        Toggle,
    }

    #[relm4::component(pub)]
    impl SimpleComponent for TogglerModel {
        type Init = bool;
        type Input = ToggleMsg;
        type Output = Msg;

        view! {
            #[root]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::ToggleButton {
                    set_label: "Toggle",
                    connect_clicked => ToggleMsg::Toggle,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Toggle: {}", model.toggle),
                    set_margin_all: 5,
                }
            }
        }

        fn init(
            init: Self::Init,
            root: Self::Root,
            sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> {
            let model = TogglerModel { toggle: init };

            let widgets = view_output!();

            ComponentParts { model, widgets }
        }

        fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
            match msg {
                ToggleMsg::Toggle => {
                    self.toggle = !self.toggle;
                }
            }
        }
    }
}
