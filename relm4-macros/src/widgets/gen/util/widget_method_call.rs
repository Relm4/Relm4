use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

use crate::widgets::WidgetMethodCall;

impl ToTokens for WidgetMethodCall {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let path = &self.path;
        let turbofish = &self.turbofish;
        tokens.extend(quote! { #path #turbofish });
    }
}
