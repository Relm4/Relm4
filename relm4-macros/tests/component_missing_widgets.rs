use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

pub struct TestComponent;

#[relm4_macros::component(pub)]
impl SimpleComponent for TestComponent {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {}
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
}

fn main() {}
