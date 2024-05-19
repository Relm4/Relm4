use relm4::prelude::*;

struct AppModel;

#[relm4_macros::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {}
    }

    #[allow(unreachable_code)]
    fn init(
        _: (),
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = AppModelWidgets {};

        ComponentParts { model, widgets }
    }
}

fn main() {}
