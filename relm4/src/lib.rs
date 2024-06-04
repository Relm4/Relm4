//! An idiomatic GUI library inspired by Elm and based on gtk4-rs.
//!
//! Docs of related crates:
//! [relm4](https://docs.rs/relm4)
//! | [relm4-macros](https://docs.rs/relm4_macros)
//! | [relm4-components](https://docs.rs/relm4_components)
//! | [relm4-css](https://docs.rs/relm4-css)
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
    unused_qualifications,
    clippy::cargo,
    clippy::must_use_candidate,
    clippy::used_underscore_binding
)]
#![allow(clippy::multiple_crate_versions)]
// Configuration for doc builds on the nightly toolchain.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod app;
mod channel;
mod extensions;
pub(crate) mod late_initialization;
mod runtime_util;

pub mod abstractions;
pub mod actions;
pub mod binding;
pub mod component;
pub mod factory;
pub mod loading_widgets;
pub mod shared_state;
pub mod typed_view;

pub use channel::ComponentSender;
pub use channel::*;
pub use component::worker::{Worker, WorkerController, WorkerHandle};
pub use component::{
    Component, ComponentBuilder, ComponentController, ComponentParts, Controller, MessageBroker,
    SimpleComponent,
};
pub use extensions::*;
pub use shared_state::{Reducer, Reducible, SharedState};
pub use shutdown::ShutdownReceiver;

pub use app::RelmApp;
pub use tokio::task::JoinHandle;

use gtk::prelude::{Cast, IsA};
use once_cell::sync::{Lazy, OnceCell};
use runtime_util::{GuardedReceiver, RuntimeSenders, ShutdownOnDrop};
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

/// Re-export of relm4-css
#[cfg(feature = "css")]
#[cfg_attr(docsrs, doc(cfg(feature = "css")))]
pub use relm4_css as css;

// Re-exports
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use relm4_macros::*;

/// Re-export of libadwaita
#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
pub use adw;

/// Re-export of libpanel
#[cfg(feature = "libpanel")]
#[cfg_attr(docsrs, doc(cfg(feature = "libpanel")))]
pub use panel;

pub use once_cell;
pub use tokio;

thread_local! {
    static MAIN_APPLICATION: Cell<Option<gtk::Application>> = Cell::default();
}

fn set_main_application(app: impl IsA<gtk::Application>) {
    MAIN_APPLICATION.with(move |cell| cell.set(Some(app.upcast())));
}

fn init() {
    gtk::init().unwrap();
    #[cfg(feature = "libadwaita")]
    adw::init().unwrap();
}

/// Returns the global [`gtk::Application`] that's used internally
/// by [`RelmApp`].
///
/// Retrieving this value can be useful for graceful shutdown
/// by calling [`ApplicationExt::quit()`][gtk::prelude::ApplicationExt::quit] on it.
///
/// Note: The global application can be overwritten by calling
/// [`RelmApp::from_app()`].
#[must_use]
pub fn main_application() -> gtk::Application {
    #[cfg(feature = "libadwaita")]
    fn new_application() -> gtk::Application {
        adw::Application::default().upcast()
    }

    #[cfg(not(feature = "libadwaita"))]
    fn new_application() -> gtk::Application {
        gtk::Application::default()
    }

    MAIN_APPLICATION.with(|cell| {
        let app = cell.take().unwrap_or_else(new_application);
        cell.set(Some(app.clone()));
        app
    })
}

#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
/// Returns the global [`adw::Application`] that's used internally
/// by [`RelmApp`] if the `libadwaita` feature is enabled.
///
/// Note: The global application can be overwritten by calling
/// [`RelmApp::from_app()`].
#[must_use]
pub fn main_adw_application() -> adw::Application {
    main_application().downcast().unwrap()
}

/// Spawns a thread-local future on GLib's executor, for non-[`Send`] futures.
pub fn spawn_local<F, Out>(func: F) -> gtk::glib::JoinHandle<Out>
where
    F: Future<Output = Out> + 'static,
    Out: 'static,
{
    gtk::glib::MainContext::ref_thread_default().spawn_local(func)
}

/// Spawns a thread-local future on GLib's executor, for non-[`Send`] futures.
pub fn spawn_local_with_priority<F, Out>(
    priority: gtk::glib::Priority,
    func: F,
) -> gtk::glib::JoinHandle<Out>
where
    F: Future<Output = Out> + 'static,
    Out: 'static,
{
    gtk::glib::MainContext::ref_thread_default().spawn_local_with_priority(priority, func)
}

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(*RELM_THREADS.get_or_init(|| 1))
        .max_blocking_threads(*RELM_BLOCKING_THREADS.get_or_init(|| 512))
        .build()
        .unwrap()
});

/// Spawns a [`Send`]-able future to the shared component runtime.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    RUNTIME.spawn(future)
}

/// Spawns a blocking task in a background thread pool.
pub fn spawn_blocking<F, R>(func: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    RUNTIME.spawn_blocking(func)
}

/// Sets a custom global stylesheet, with the given priority.
///
/// The priority can be any value, but GTK [includes some][style-providers] that you can use.
///
/// [style-providers]: https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/index.html?search=const%3ASTYLE_PROVIDER&filter-crate=gtk4#constants
pub fn set_global_css_with_priority(style_data: &str, priority: u32) {
    let display = gtk::gdk::Display::default().unwrap();
    let provider = gtk::CssProvider::new();
    #[allow(deprecated)]
    provider.load_from_data(style_data);

    #[allow(deprecated)]
    gtk::StyleContext::add_provider_for_display(&display, &provider, priority);
}
/// Sets a custom global stylesheet.
pub fn set_global_css(style_data: &str) {
    set_global_css_with_priority(style_data, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}

/// Sets a custom global stylesheet from a file, with the given priority.
///
/// If the file doesn't exist a [`tracing::error`] message will be emitted and
/// an [`std::io::Error`] will be returned.
///
/// The priority can be any value, but GTK [includes some][style-providers] that you can use.
///
/// [style-providers]: https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/index.html?search=const%3ASTYLE_PROVIDER&filter-crate=gtk4#constants
pub fn set_global_css_from_file_with_priority<P: AsRef<std::path::Path>>(
    path: P,
    priority: u32,
) -> Result<(), std::io::Error> {
    std::fs::read_to_string(path)
        .map(|bytes| set_global_css_with_priority(&bytes, priority))
        .map_err(|err| {
            tracing::error!("Couldn't load global CSS from file: {}", err);
            err
        })
}

/// Sets a custom global stylesheet from a file.
///
/// If the file doesn't exist a [`tracing::error`] message will be emitted and
/// an [`std::io::Error`] will be returned.
pub fn set_global_css_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<(), std::io::Error> {
    set_global_css_from_file_with_priority(path, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION)
}
