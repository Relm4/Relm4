use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(default)]
struct TestComponent;

#[relm4_macros::component]
impl SimpleComponent for TestComponent {
    type Init = ();
    type Input = ();
    type Output = ();
    type Widgets = TestWidgets;

    view! {
        gtk::Window {}
    }

    fn init(
        _init_param: (),
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::default();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    // Incorrect impl
    fn 
}

fn main() {}
