use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, token};

use super::{PropertyName, ReturnedWidget, Widget, WidgetTemplateAttr};

/// Utility methods and functions.
mod util;

/// Generate struct fields.
mod struct_fields;

/// Fields of the returned widget struct.
mod return_fields;

mod assign;
mod conditional_init;
mod destructure_fields;
mod error;
mod init;
mod update_view;

impl Widget {
    pub(super) fn widget_assignment(&self) -> TokenStream2 {
        let w_name = &self.name;

        let ref_token = &self.ref_token;
        let deref_token = &self.deref_token;
        let template_deref =
            (self.template_attr == WidgetTemplateAttr::Template).then(token::Star::default);

        let out_stream = quote! { #ref_token #deref_token #template_deref #w_name };

        if let Some(wrapper) = &self.assign_wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }
}
