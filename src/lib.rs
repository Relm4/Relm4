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

pub fn spawn_future<F: futures_core::future::Future<Output = ()> + Send + 'static>(f: F) {
    gtk::glib::MainContext::ref_thread_default().spawn(f);
}
