use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

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
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        // This would compile before 0.5.0-beta.3
        // but shouldn't because the window isn't used
        // and doesn't need to be part of the widgets
        // struct.
        let _window = &widgets._gtk_window_0;

        ComponentParts { model, widgets }
    }
}

fn main() {}
