use gtk::prelude::{BoxExt, GtkWindowExt, OrientableExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, WidgetTemplate};

#[relm4_macros::widget_template]
impl WidgetTemplate for CustomBox {
    view! {
        gtk::Box {
            set_margin_all: 5,
            set_spacing: 5,

            #[name = "label"]
            gtk::Label {
                set_label: "Is it working?",
            }
        }
    }
}

#[relm4_macros::widget_template]
impl WidgetTemplate for CustomWindow {
    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            #[template]
            #[name = "custom_box"]
            CustomBox {
                set_orientation: gtk::Orientation::Vertical,
            }
        }
    }
}

#[derive(Default)]
struct App {
    counter: u8,
}

#[derive(Debug)]
enum AppMsg {}

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[template]
        CustomWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            // Nested template child
            #[template_child]
            custom_box.label {
                #[watch]
                set_label: "It works!",
            },
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { counter };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: AppMsg, _sender: ComponentSender<Self>) {
        self.counter += 1;
    }
}
