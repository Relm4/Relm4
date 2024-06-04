//! Static definitions for Adwaita's style classes and colors.
//!
//! Most of the functionality requires Adwaita being active in your
//! application, though you may have luck using them with just GTK.
//!
//! - Available classes: <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.2/style-classes.html>.
//! - Available colors: <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.2/named-colors.html#palette-colors>.
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

pub use classes::*;
pub use colors::*;

/// Adwaita's CSS classes.
pub mod classes {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

/// Adwaita's CSS colors.
pub mod colors {
    include!(concat!(env!("OUT_DIR"), "/colors.rs"));
}
