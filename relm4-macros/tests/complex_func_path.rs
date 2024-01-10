use gtk::prelude::{GtkWindowExt, OrientableExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Default)]
struct App;

trait TestType {
    type Test;
}

impl TestType for App {
    type Test = gtk::Box;
}

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            gtk::Box::default() -> <App as TestType>::Test {
                set_orientation: gtk::Orientation::Vertical,
            },
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: (), _sender: ComponentSender<Self>) {}
}
