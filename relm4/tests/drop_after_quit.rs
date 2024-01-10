use std::sync::atomic::{AtomicBool, Ordering};

use gtk::prelude::*;
use relm4::{main_application, prelude::*};

static APP_DROPPED: AtomicBool = AtomicBool::new(false);

struct App;

impl Drop for App {
    fn drop(&mut self) {
        APP_DROPPED.store(true, Ordering::SeqCst);
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
        root: Self::Root,
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
fn drop_after_quit() {
    let app = RelmApp::new("relm4.test.dropAfterQuit");
    app.run::<App>(());
    assert!(APP_DROPPED.load(Ordering::SeqCst));
}
