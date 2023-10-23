use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Default)]
struct TestComponent {
    counter: u8,
}

#[relm4_macros::component]
impl SimpleComponent for TestComponent {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {}
    }

    fn pre_view() {
        if model.counter == 0 {
            return;
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {}
