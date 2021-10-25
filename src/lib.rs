//! An idiomatic GUI library inspired by Elm and based on gtk4-rs

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/AaronErhardt/relm4/main/assets/Relm_logo.svg"
)]

pub mod actions;
mod app;
mod component;
pub mod drawing;
pub mod factory;
mod msg_handler;
mod traits;
pub mod util;
mod worker;

pub use app::RelmApp;
pub use component::RelmComponent;
pub use msg_handler::RelmMsgHandler;
pub use traits::*;
pub use util::widget_plus::WidgetPlus;
pub use worker::*;

use fragile::Fragile;
use once_cell::sync::OnceCell;

static APP: OnceCell<Fragile<Application>> = OnceCell::new();

pub use gtk;
pub use gtk::glib::Sender;

#[cfg(feature = "libadwaita")]
type Application = adw::Application;

#[cfg(not(feature = "libadwaita"))]
type Application = gtk::Application;

#[cfg(feature = "tokio-rt")]
#[cfg_attr(doc, doc(cfg(feature = "tokio-rt")))]
/// Re-export of [`async_trait::async_trait`]
pub use async_trait::async_trait;

#[must_use]
/// Returns the application created by [`RelmApp::new`].
///
/// # Panics
///
/// This function panics if [`RelmApp::new`] wasn't called before
/// or this function is not called on the thread that also called [`RelmApp::new`].
pub fn gtk_application() -> Application {
    APP.get()
        .expect("The gloabl gtk application hasn't been initialized yet")
        .try_get()
        .expect("The global gtk application can only be read from the main thread")
        .clone()
}

/// Sets a custom global stylesheet.
///
/// # Panics
///
/// This function panics if [`RelmApp::new`] wasn't called before
/// or this function is not called on the thread that also called [`RelmApp::new`].
pub fn set_global_css(style_data: &[u8]) {
    let display = gtk::gdk::Display::default().unwrap();
    let provider = gtk::CssProvider::new();
    provider.load_from_data(style_data);
    gtk::StyleContext::add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Sets a custom global stylesheet from a file.
///
/// If the file doesn't exist a [`log::error`] message will be emitted.
///
/// # Panics
///
/// This function panics if [`RelmApp::new`] wasn't called before
/// or this function is not called on the thread that also called [`RelmApp::new`].
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

/// Spawns a future on the main thread in the main event loop.
///
/// # Panics
///
/// This function itself doesn't panic but it might panic if you run futures that
/// expect the tokio runtime. Use the tokio-rt feature and an `AsyncComponent` for this instead.
pub fn spawn_future<F: futures_core::future::Future<Output = ()> + Send + 'static>(f: F) {
    gtk::glib::MainContext::ref_thread_default().spawn(f);
}

/// A short macro for conveniently sending messages.
///
/// The message is sent using the sender and the [`Result`] is unwrapped automatically.
#[macro_export]
macro_rules! send {
    ($sender:ident, $msg:expr) => {
        $sender.send($msg).expect("Receiver was dropped!")
    };
}
