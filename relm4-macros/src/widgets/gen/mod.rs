use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

use super::{PropertyName, ReturnedWidget, Widget};

/// Utility methods and functions.
mod util;

/// Generate struct fields.
mod struct_fields;

/// Fields of the returned widget struct.
mod return_fields;

mod assign;
mod connect_signals;
mod destructure_fields;
mod init;
mod update_view;

impl Widget {
    pub fn widget_assignment(&self) -> TokenStream2 {
        let w_name = &self.name;

        let ref_token = &self.ref_token;
        let deref_token = &self.deref_token;

        let out_stream = quote! { #ref_token #deref_token #w_name };

        if let Some(wrapper) = &self.wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }
}
