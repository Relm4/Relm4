mod gtk;

#[cfg(feature = "libadwaita")]
mod libadwaita;

#[cfg(feature = "libpanel")]
mod libpanel;

#[cfg(test)]
mod tests;

/// Trait used for factories to interact with widgets.
pub mod traits;

pub use traits::*;
