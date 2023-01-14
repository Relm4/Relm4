mod additional_fields;
mod args;
pub mod attrs;
pub mod menu;
pub mod token_streams;
mod util;
pub mod visitors;
pub mod widgets;

pub fn gtk_import() -> syn::Path {
    if cfg!(feature = "relm4") {
        util::strings_to_path(&["relm4", "gtk"])
    } else {
        util::strings_to_path(&["gtk"])
    }
}
