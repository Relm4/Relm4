//! An idiomatic GUI library inspired by Elm and based on gtk4-rs.

#![doc(html_logo_url = "https://raw.githubusercontent.com/Relm4/Relm4/main/assets/Relm_logo.svg")]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/Relm4/Relm4/main/assets/Relm_logo.svg"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
// Configuration for doc builds on the nightly toolchain.
#![cfg_attr(dox, feature(doc_cfg))]

pub mod actions;
mod app;
mod channel;

/// Components are smaller mostly independent parts of
/// your application.
pub mod component;

pub mod drawing;
mod extensions;
pub mod factory;

/// Cancellation mechanism used by Relm4.
pub mod shutdown;

/// Shared state that can be accessed by many components.
pub mod shared_state;

pub mod util;

/// A simpler version of components that does work
/// in the background.
pub mod worker;

pub use self::channel::{channel, Receiver, Sender};
pub use self::component::{
    Component, ComponentBuilder, ComponentController, ComponentParts, ComponentSender, Controller,
    OnDestroy, SimpleComponent,
};
pub use self::extensions::*;
pub use self::shared_state::SharedState;
pub use self::shutdown::ShutdownReceiver;
pub use self::worker::{Worker, WorkerController, WorkerHandle};

pub use app::RelmApp;
pub use tokio::task::JoinHandle;
pub use util::{WidgetPlus, WidgetRef};

use once_cell::sync::OnceCell;
use std::future::Future;
use tokio::runtime::Runtime;

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

#[cfg(feature = "libpanel")]
/// Re-export of libpanel
pub use panel;

#[cfg(feature = "libadwaita")]
type Application = adw::Application;

#[cfg(not(feature = "libadwaita"))]
type Application = gtk::Application;

pub use tokio;

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
            log::error!("Couldn't load global CSS from file: {}", err);
        }
    }
}

/// Spawns a thread-local future on GLib's executor, for non-[`Send`] futures.
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

/// Spawns a [`Send`]-able future to the shared component runtime.
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
#[deprecated(since = "0.5.0", note = "Use `sender.input(msg)` instead.")]
macro_rules! send {
    ($sender:expr, $msg:expr) => {
        $sender.input($msg)
    };
}
