use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::gen::Property;

impl Property {
    pub fn args_stream(&self) -> Option<TokenStream2> {
        self.args.as_ref().map(|args| quote! { ,#args })
    }
}
