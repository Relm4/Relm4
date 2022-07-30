use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

struct TestComponent;

#[relm4_macros::component]
impl SimpleComponent for TestComponent {
    type InitParams = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {}
    }

    fn init(
        init_param: (),
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {}
