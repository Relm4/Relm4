use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Ident, Path};

use crate::widgets::{AssignProperty, AssignPropertyAttr, Property, PropertyName, PropertyType};

impl Property {
    pub fn update_view_stream(&self, stream: &mut TokenStream2, w_name: &Ident, relm4_path: &Path) {
        if let PropertyType::Assign(assign) = &self.ty {
            assign.update_view_stream(stream, &self.name, w_name, relm4_path)
        }
    }
}

impl AssignProperty {
    pub fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        match &self.attr {
            AssignPropertyAttr::None => (),
            AssignPropertyAttr::Watch => {
                self.assign_stream(stream, p_name, w_name, relm4_path);
            }
            AssignPropertyAttr::Track(track_stream) => {
                let mut assign_stream = TokenStream2::new();
                self.assign_stream(&mut assign_stream, p_name, w_name, relm4_path);

                stream.extend(quote_spanned! {
                    track_stream.span() => if #track_stream {
                        #assign_stream
                    }
                });
            }
        }
    }
}
