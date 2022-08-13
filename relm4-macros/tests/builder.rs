use gtk::prelude::GtkWindowExt;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Default)]
struct AppModel;

#[relm4_macros::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = ();
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                gtk::Builder::from_string("<Label id=\"label\"></Label>")
                    .object::<gtk::Label>("label")
                    .unwrap() -> gtk::Label {},
            },
        }
    }

    fn init(
        _init_params: Self::Init,
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: (), _sender: ComponentSender<Self>) {}
}

fn assert_impls_debug<T: std::fmt::Debug>() {}

#[test]
fn assert_widgets_impl_debug() {
    assert_impls_debug::<AppWidgets>();
}
