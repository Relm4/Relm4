#![deny(unreachable_code)]

use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

struct TestComponent;

#[relm4_macros::component]
impl SimpleComponent for TestComponent {
    type InitParams = ();
    type Input = ();
    type Output = ();
    type Widgets = TestWidgets;

    view! {
        gtk::Window {}
    }

    fn pre_view() {
        return;
    }

    fn init(
        _init_param: (),
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {}
