#![doc(
    html_logo_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]

mod app;
mod component;
pub mod default_widgets;
pub mod generator;
mod traits;
mod worker;

pub use app::RelmApp;
pub use component::RelmComponent;
pub use traits::*;
pub use worker::*;

pub use gtk::glib::Sender;

#[cfg(feature = "tokio-rt")]
pub use async_trait::async_trait;

pub fn set_global_css(style_data: &[u8]) {
    let display = gtk::gdk::Display::default().unwrap();
    let provider = gtk::CssProvider::new();
    provider.load_from_data(style_data);
    gtk::StyleContext::add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

pub fn set_global_css_from_file<P: AsRef<std::path::Path>>(path: P) {
    match std::fs::read(path) {
        Ok(bytes) => {
            set_global_css(&bytes);
        }
        Err(err) => {
            log::error!("Couln't load global CSS from file: {}", err);
        }
    }
}

pub fn spawn_future<F: futures_core::future::Future<Output = ()> + Send + 'static>(f: F) {
    gtk::glib::MainContext::ref_thread_default().spawn(f);
}

#[macro_export]
macro_rules! send {
    ($sender:ident, $msg:expr) => {
        $sender.clone().send($msg).unwrap()
    };
}
