use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, Path};

use crate::widgets::Properties;

impl Properties {
    pub(super) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        for prop in &self.properties {
            prop.assign_stream(stream, w_name, is_conditional, relm4_path);
        }
    }
}
