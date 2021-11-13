use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::Widget;

impl Widget {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;
        stream.extend(quote! { #name, });
    }
}
