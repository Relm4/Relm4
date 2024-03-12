//! A module for convenient abstractions over gtk-rs.

pub mod drawing;

#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
mod toaster;

pub use drawing::{DrawContext, DrawHandler};

#[cfg(feature = "libadwaita")]
#[cfg_attr(docsrs, doc(cfg(feature = "libadwaita")))]
pub use toaster::Toaster;
