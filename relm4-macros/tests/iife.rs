use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

struct App;

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            gtk::Label {
                #[watch]
                set_label: &format!("Counter: {counter}"),
            }
        }
    }

    fn pre_view() {
        // Only works if pre_view isn't wrapped inside an IIFE
        // because the local variable counter is used in the
        // update_view method.
        let counter = 1;
    }

    fn init(
        _counter: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let counter = 1;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
