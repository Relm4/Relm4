use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

#[derive(Default)]
struct TestComponent;

#[relm4_macros::component]
impl SimpleComponent for TestComponent {
    type Init = ();
    type Input = ();
    type Output = ();
    
    view! {
        gtk::Window {}
    }

    fn init(
        _init: Self::Init,
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
