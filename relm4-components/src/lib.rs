//! Collection of reusable and easily configurable components for Relm4.

#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Relm4/relm4/main/assets/Relm_logo.svg"
)]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/Relm4/relm4/main/assets/Relm_logo.svg"
)]

pub mod alert;
pub mod open_button;
pub mod open_dialog;
pub mod save_dialog;
mod traits;

pub use traits::*;
