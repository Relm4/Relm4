use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::gen::Property;

impl Property {
    pub fn args_stream(&self) -> TokenStream2 {
        if let Some(args) = &self.args {
            quote! { ,#args }
        } else {
            TokenStream2::new()
        }
    }
}
