use gtk::prelude::*;
use relm4::{main_application, prelude::*};

struct App;

impl Drop for App {
    fn drop(&mut self) {
        panic!("App dropped");
    }
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {}
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App;
        let widgets = view_output!();

        sender.input(());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {
        main_application().quit();
    }
}

#[test]
#[should_panic = "App dropped"]
fn drop_after_quit() {
    let app = RelmApp::new("relm4.test.dropAfterQuit");
    app.run::<App>(());
}
