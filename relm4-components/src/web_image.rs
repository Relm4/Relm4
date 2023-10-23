//! Reusable and easily configurable component for loading images from the web.

use std::collections::VecDeque;
use std::fmt::Debug;

use relm4::gtk::prelude::{BoxExt, Cast, WidgetExt};
use relm4::{gtk, Component, ComponentParts, ComponentSender};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Reusable component for loading images from the web.
pub struct WebImage {
    current_id: usize,
    current_widget: gtk::Widget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Load or unload a web image.
pub enum WebImageMsg {
    /// Load an image from an url.
    LoadImage(String),
    /// Unload the current image.
    Unload,
}

impl Component for WebImage {
    type CommandOutput = Option<(usize, VecDeque<u8>)>;
    type Input = WebImageMsg;
    type Output = ();
    type Init = String;
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::default()
    }

    fn init(
        url: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widget = gtk::Box::default();
        root.append(&widget);
        let current_widget = Self::set_spinner(root, widget.upcast_ref());

        let model = Self {
            current_id: 0,
            current_widget,
        };

        model.load_image(&sender, url);

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        self.current_widget = Self::set_spinner(root, &self.current_widget);
        self.current_id = self.current_id.wrapping_add(1);

        match input {
            WebImageMsg::LoadImage(url) => {
                self.load_image(&sender, url);
            }
            WebImageMsg::Unload => (),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        if let Some((id, data)) = message {
            if id == self.current_id {
                if let Some(img) = Self::generate_image(data) {
                    self.current_widget = Self::set_image(root, &self.current_widget, &img);
                    sender.output(()).ok();
                }
            }
        }
    }
}

impl WebImage {
    #[must_use]
    fn set_spinner(root: &<Self as Component>::Root, widget: &gtk::Widget) -> gtk::Widget {
        root.remove(widget);
        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        }
        spinner.upcast()
    }

    #[must_use]
    fn set_image(
        root: &<Self as Component>::Root,
        widget: &gtk::Widget,
        img: &gtk::Image,
    ) -> gtk::Widget {
        root.remove(widget);
        relm4::view! {
            #[local_ref]
            root -> gtk::Box {
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,

                #[local_ref]
                img -> gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                }
            }
        }
        img.clone().upcast()
    }

    fn load_image(&self, sender: &ComponentSender<Self>, url: String) {
        sender.oneshot_command(Self::get_img_data(self.current_id, url));
    }

    fn generate_image(data: VecDeque<u8>) -> Option<gtk::Image> {
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_read(data).ok()?;
        Some(gtk::Image::from_pixbuf(Some(&pixbuf)))
    }

    async fn get_img_data(id: usize, url: String) -> Option<(usize, VecDeque<u8>)> {
        let response = reqwest::get(url).await.ok()?;
        let bytes = response.bytes().await.ok()?;
        Some((id, bytes.into_iter().collect()))
    }
}
