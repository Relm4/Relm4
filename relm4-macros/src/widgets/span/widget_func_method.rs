use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::widgets::WidgetFuncMethod;

impl ToTokens for WidgetFuncMethod {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let WidgetFuncMethod {
            ident,
            turbofish,
            args,
        } = &self;
        tokens.extend(if let Some(args) = args {
            quote! {
                #ident #turbofish (#args)
            }
        } else {
            quote! {
                #ident #turbofish
            }
        });
    }
}
