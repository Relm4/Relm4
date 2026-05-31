use relm4::gtk::prelude::*;
use relm4::prelude::*;
pub struct App(Controller<Inner>);

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        window = gtk::Window {
            set_title: Some("Conditional Root Example"),
            set_default_size: (200, 150),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,
                set_align: gtk::Align::Center,

                #[local_ref]
                inner -> gtk::Stack {},

                gtk::Button {
                    set_label: "Toggle",
                    connect_clicked => (),
                }
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self(Inner::builder().launch(()).detach());
        let inner = model.0.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {
        self.0
            .sender()
            .send(())
            .expect("Failed to send message to Inner component!");
    }
}

pub struct Inner {
    state: bool,
    counter: usize,
}

#[relm4::component(pub)]
impl SimpleComponent for Inner {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        if model.state {
            gtk::Label {
                #[watch]
                set_label: &format!("Toggled {} times", model.counter),
            }
        } else {
            gtk::Label {
                set_label: "False",
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            state: true,
            counter: 0,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {
        self.state = !self.state;
        self.counter += 1;
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.conditional_root");
    app.run::<App>(());
}
