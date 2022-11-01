//! An idiomatic GUI library inspired by Elm and based on gtk4-rs.
//!
//! The docs are available in two versions.
//! Use the [stable docs](https://docs.rs/relm4) if you want get information about a version that was already published.
//! Visit the [nightly docs](https://relm4.org/docs/next/relm4/) if are trying out the newest but possibly unstable version of the crate.
//!
//! Docs of related crates:
//! [relm4](../relm4/index.html)
//! | [relm4-macros](../relm4_macros/index.html)
//! | [relm4-components](../relm4_components/index.html)
//! | [gtk4-rs](https://gtk-rs.org/gtk4-rs/git/docs)
//! | [gtk-rs-core](https://gtk-rs.org/gtk-rs-core/git/docs)
//! | [libadwaita-rs](https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/git/docs/libadwaita)
//! | [libpanel-rs](https://world.pages.gitlab.gnome.org/Rust/libpanel-rs/git/docs/libpanel)
//!
//! [GitHub](https://github.com/Relm4/Relm4)
//! | [Website](https://relm4.org)
//! | [Book](https://relm4.org/book/stable/)
//! | [Blog](https://relm4.org/blog)

#![doc(html_logo_url = "https://relm4.org/icons/relm4_logo.svg")]
#![doc(html_favicon_url = "https://relm4.org/icons/relm4_org.svg")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::cargo,
    clippy::must_use_candidate
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

mod sender;

/// Cancellation mechanism used by Relm4.
pub mod shutdown;

/// Shared state that can be accessed by many components.
pub mod shared_state;

pub(crate) mod late_initialization;
/// A simpler version of components that does work
/// in the background.
pub mod worker;

pub use channel::{channel, Receiver, Sender};
pub use component::{
    Component, ComponentBuilder, ComponentController, ComponentParts, Controller, MessageBroker,
    OnDestroy, SimpleComponent,
};
pub use extensions::*;
pub use sender::ComponentSender;
pub use shared_state::SharedState;
pub use shutdown::ShutdownReceiver;
pub use worker::{Worker, WorkerController, WorkerHandle};

pub use app::RelmApp;
pub use tokio::task::JoinHandle;

use gtk::prelude::*;
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::future::Future;
use tokio::runtime::Runtime;

/// Defines how many threads that Relm4 should use for background tasks.
///
/// NOTE: The default thread count is 1.
pub static RELM_THREADS: OnceCell<usize> = OnceCell::new();

/// Defines the maximum number of background threads to spawn for handling blocking tasks.
///
/// NOTE: The default max is 512.
pub static RELM_BLOCKING_THREADS: OnceCell<usize> = OnceCell::new();

pub mod prelude;

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

pub use once_cell;
pub use tokio;

thread_local! {
    static MAIN_APPLICATION: Cell<Option<gtk::Application>> = Cell::default();
}

fn set_main_application(app: impl IsA<gtk::Application>) {
    MAIN_APPLICATION.with(move |cell| cell.set(Some(app.upcast())));
}

#[cfg(feature = "libadwaita")]
fn new_application() -> gtk::Application {
    adw::Application::default().upcast()
}

#[cfg(not(feature = "libadwaita"))]
fn new_application() -> gtk::Application {
    gtk::Application::default()
}

/// Returns the global [`gtk::Application`] that's used internally
/// by [`RelmApp`].
///
/// Retrieving this value can be useful for graceful shutdown
/// by calling [`ApplicationExt::quit()`][gtk::prelude::ApplicationExt::quit] on it.
///
/// Note: The global application can be overwritten by calling
/// [`RelmApp::with_app()`].
#[must_use]
pub fn main_application() -> gtk::Application {
    MAIN_APPLICATION.with(|cell| {
        let app = cell.take().unwrap_or_else(new_application);
        cell.set(Some(app.clone()));
        app
    })
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
/// If the file doesn't exist a [`tracing::error`] message will be emitted.
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
            tracing::error!("Couldn't load global CSS from file: {}", err);
        }
    }
}

/// Spawns a thread-local future on GLib's executor, for non-[`Send`] futures.
pub fn spawn_local<F: Future<Output = ()> + 'static>(func: F) -> gtk::glib::SourceId {
    gtk::glib::MainContext::ref_thread_default().spawn_local(func)
}

/// Spawns a thread-local future on GLib's executor, for non-[`Send`] futures.
pub fn spawn_local_with_priority<F: Future<Output = ()> + 'static>(
    priority: gtk::glib::Priority,
    func: F,
) -> gtk::glib::SourceId {
    gtk::glib::MainContext::ref_thread_default().spawn_local_with_priority(priority, func)
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

/// Spawns a blocking task in a background thread pool.
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
#[deprecated(since = "0.5.0-beta.1", note = "Use `sender.input(msg)` instead.")]
macro_rules! send {
    ($sender:expr, $msg:expr) => {
        $sender.input($msg)
    };
}
