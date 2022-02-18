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
mod extensions;
pub mod factory;

/// Cancellation mechanism used by Relm
pub mod shutdown;

pub mod util;
mod worker;

pub use self::component::*;
pub use self::extensions::*;
pub use self::worker::*;
pub use app::RelmApp;
pub use tokio::task::JoinHandle;
pub use util::widget_plus::WidgetPlus;

use once_cell::sync::OnceCell;
use std::future::Future;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Re-export of `tokio::sync::mpsc::UnboundedSender`.
pub type Sender<T> = mpsc::UnboundedSender<T>;

/// Re-export of `tokio::sync::mpsc::UnboundedReceiver`.
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

/// Defines how many threads that Relm should use for background tasks.
///
/// NOTE: The default thread count is 1.
pub static RELM_THREADS: OnceCell<usize> = OnceCell::new();

/// Defines the maximum number of background threads to spawn for handling blocking tasks.
///
/// NOTE: The default max is 512.
pub static RELM_BLOCKING_THREADS: OnceCell<usize> = OnceCell::new();

/// Re-export of gtk4
pub use gtk;

// Re-exports
#[cfg(feature = "macros")]
pub use relm4_macros::*;

#[cfg(feature = "libadwaita")]
/// Re-export of libadwaita
pub use adw;

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

/// Spawns a thread-local future on GLib's executor, for non-Send futures.
pub fn spawn_local<F: Future<Output = ()> + 'static>(func: F) -> gtk::glib::SourceId {
    gtk::glib::MainContext::ref_thread_default().spawn_local(func)
}

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(*RELM_THREADS.get_or_init(|| 1))
            .max_blocking_threads(*RELM_BLOCKING_THREADS.get_or_init(|| 512))
            .build()
            .unwrap()
    })
}

/// Spawns a `Send`-able future to the shared component runtime.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(future)
}

/// Spawns a blocking task in a background thread pool
pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    runtime().spawn_blocking(func)
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
