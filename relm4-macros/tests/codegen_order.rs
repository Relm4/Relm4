use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[derive(Default)]
struct App;

#[relm4_macros::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        mut Vec::<Vec<u8>> {
            push = mut Vec {
                push: 0,
                push: 1,
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
