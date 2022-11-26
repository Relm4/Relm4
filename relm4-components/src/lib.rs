//! Collection of reusable and easily configurable components for Relm4.
//!
//! The docs are available in two versions.
//! Use the [stable docs](https://relm4.org/docs/stable/relm4_components/) if you want get information about a version that was already published.
//! Visit the [nightly docs](https://relm4.org/docs/next/relm4_components/) if are trying out the newest but possibly unstable version of the crate.
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
    unused_qualifications,
    clippy::cargo,
    clippy::must_use_candidate
)]

pub mod alert;
pub mod open_button;
pub mod open_dialog;
pub mod save_dialog;
pub mod simple_combo_box;
