use gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent, gtk,
};
use relm4_components::web_image::{WebImage, WebImageMsg};

const IMAGES: &[&str] = &[
    "https://raw.githubusercontent.com/Relm4/Relm4/main/assets/Relm_logo_with_text.png",
    "https://raw.githubusercontent.com/Relm4/Relm4/main/assets/Relm_logo.png",
    "https://raw.githubusercontent.com/gtk-rs/gtk-rs.github.io/master/logo/gtk-rs.ico",
    "https://avatars.githubusercontent.com/u/5430905",
];

#[derive(Debug)]
enum AppMsg {
    Next,
    Unload,
}

struct App {
    image: Controller<WebImage>,
    idx: usize,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_default_size: (300, 300),

            #[wrap(Some)]
            set_titlebar = &gtk::HeaderBar {
                pack_start = &gtk::Button {
                    set_label: "Next image",
                    connect_clicked => AppMsg::Next,
                },
                pack_start = &gtk::Button {
                    set_label: "Unload image",
                    connect_clicked => AppMsg::Unload,
                }
            },

            gtk::Box {
                #[local_ref]
                image -> gtk::Box {}
            }
        }
    }

    fn update(&mut self, msg: Self::Input, _: ComponentSender<Self>) {
        match msg {
            AppMsg::Next => {
                self.idx = (self.idx + 1) % IMAGES.len();
                self.image
                    .emit(WebImageMsg::LoadImage(IMAGES[self.idx].to_owned()));
            }
            AppMsg::Unload => self.image.emit(WebImageMsg::Unload),
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let image = WebImage::builder().launch(IMAGES[0].to_owned()).detach();
        let model = App { image, idx: 0 };

        let image = model.image.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.open_button");
    app.run::<App>(());
}
