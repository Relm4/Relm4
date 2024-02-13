//! Collection of reusable and easily configurable components for Relm4.
//!
//! Docs of related crates:
//! [relm4](https://docs.rs/relm4)
//! | [relm4-macros](https://docs.rs/relm4_macros)
//! | [relm4-components](https://docs.rs/relm4_components)
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
    clippy::must_use_candidate
)]
#![allow(clippy::multiple_crate_versions)]
// Configuration for doc builds on the nightly toolchain.
#![cfg_attr(docsrs, feature(doc_cfg))]
// Ignore GTK 4.10 deprecations.
// Most deprecated features can only be replaced with new 4.10 APIs and
// we don't want to lift the minimum requirement GTK4 version for Relm4 yet.
#![allow(deprecated)]

pub mod alert;
pub mod open_button;
pub mod open_dialog;
pub mod save_dialog;
#[cfg(feature = "libadwaita")]
pub mod simple_adw_combo_row;
pub mod simple_combo_box;

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
pub mod web_image;
