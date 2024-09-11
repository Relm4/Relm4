use gtk::prelude::*;
use relm4::prelude::*;

use gtk::glib;
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::gtk::gio::{Cancellable, MemoryInputStream};


struct App {}


/// embedded logo as paintable texture
///
/// The bytes from PNG are included during build time and shipped
/// within the executable.
fn embedded_logo() -> Texture {
    let bytes = include_bytes!(".././assets/Relm_logo.png");
    let g_bytes = glib::Bytes::from(&bytes.to_vec());
    let stream = MemoryInputStream::from_bytes(&g_bytes);
    let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
    Texture::for_pixbuf(&pixbuf)
}


#[derive(Debug)]
enum Msg {}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Embedded Logo app "),
            set_default_size: (200, 200),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Image {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_paintable: Some(&embedded_logo()),
                },
            }
        }
    }

    // Initialize the component.
    fn init(
        _: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {};

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}

fn main() {
    let app = RelmApp::new("relm4.example.embedded_logo");
    app.run::<App>(0);
}
