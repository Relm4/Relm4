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
mod component;
pub mod drawing;
pub mod factory;
pub mod util;
// mod worker;

// pub use self::worker::*;
pub use component::{Component, ComponentParts, Finalized, Handle, RawComponent};
pub use util::widget_plus::WidgetPlus;

use fragile::Fragile;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc;

/// Re-export of `tokio::sync::mpsc::UnboundedSender`.
pub type Sender<T> = mpsc::UnboundedSender<T>;

/// Re-export of `tokio::sync::mpsc::UnboundedReceiver`.
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

static APP: OnceCell<Fragile<Application>> = OnceCell::new();

/// Re-export of gtk4
pub use gtk;

#[cfg(feature = "libadwaita")]
type Application = adw::Application;

#[cfg(not(feature = "libadwaita"))]
type Application = gtk::Application;

// Re-exports
#[cfg(feature = "macros")]
pub use relm4_macros::*;

#[cfg(feature = "libadwaita")]
/// Re-export of libadwaita
pub use adw;

/// Re-export of [`async_trait::async_trait`]
pub use async_trait::async_trait;

/// Forwards an event from one channel to another.
pub async fn forward<Transformer, Input, Output>(
    mut receiver: Receiver<Input>,
    sender: Sender<Output>,
    transformer: Transformer,
) where
    Transformer: (Fn(Input) -> Output) + 'static,
    Input: 'static,
    Output: 'static,
{
    while let Some(event) = receiver.recv().await {
        if sender.send(transformer(event)).is_err() {
            break;
        }
    }
}

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
pub fn spawn_future<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    gtk::glib::MainContext::ref_thread_default().spawn(f);
}

/// Spawns a thread-local future on GLib's executor, for non-Send futures.
pub fn spawn_local<F: std::future::Future<Output = ()> + 'static>(func: F) {
    gtk::glib::MainContext::ref_thread_default().spawn_local(func);
}

/// A short macro for conveniently sending messages.
///
/// The message is sent using the sender and the [`Result`] is unwrapped automatically.
#[macro_export]
macro_rules! send {
    ($sender:expr, $msg:expr) => {
        $sender.send($msg).expect("Receiver was dropped!")
    };
}
