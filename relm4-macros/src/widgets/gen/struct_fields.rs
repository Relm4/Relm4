use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Visibility;

use super::Widget;

impl Widget {
    pub fn struct_fields_stream(&self, stream: &mut TokenStream2, vis: &Option<Visibility>) {
        let name = &self.name;
        let ty = self.func.type_token_stream();

        stream.extend(quote! {
            #[allow(missing_docs)]
            #vis #name: #ty,
        });
    }
}
