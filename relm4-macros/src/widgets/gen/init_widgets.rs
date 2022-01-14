use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::Widget;

impl Widget {
    pub fn init_widgets_stream(&self, stream: &mut TokenStream2) {
        let mutability = &self.mutable;
        let name = &self.name;
        let func = self.func.func_token_stream();
        stream.extend(quote! {
            let #mutability #name = #func;
        });
    }
}
