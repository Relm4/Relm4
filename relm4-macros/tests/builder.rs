use gtk::prelude::GtkWindowExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

#[derive(Default)]
pub struct App;

#[relm4_macros::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),

            gtk::Box {
                gtk::Builder::from_string("<Label id=\"label\"></Label>")
                    .object::<gtk::Label>("label")
                    .unwrap() -> gtk::Label {},
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
