use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, Path};

use crate::widgets::Properties;

impl Properties {
    pub(super) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        for prop in &self.properties {
            prop.assign_stream(stream, w_name, relm4_path);
        }
    }
}
