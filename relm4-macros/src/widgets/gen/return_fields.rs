use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::{ReturnedWidget, Widget};

impl Widget {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;
        stream.extend(quote! { #name, });
    }
}

impl ReturnedWidget {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        if self.ty.is_some() {
            let name = &self.name;
            stream.extend(quote! {
                #name,
            });
        }
    }
}
