use gtk::prelude::*;
use relm4::{main_application, prelude::*};

struct App;

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

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        panic!("Shutdown called");
    }
}

#[test]
#[should_panic = "Shutdown called"]
fn shutdown_after_quit() {
    let app = RelmApp::new("relm4.test.shutdownAfterQuit");
    app.run::<App>(());
}
