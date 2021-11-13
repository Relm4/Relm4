use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::Ident;

use crate::widgets::gen::PropertyType;
impl PropertyType {
    pub fn return_assign_tokens(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();

        if let PropertyType::Widget(widget) = self {
            if let Some(returned_widget) = &widget.returned_widget {
                let name = if let Some(name) = &returned_widget.name {
                    name.clone()
                } else {
                    Ident::new("placeholder", Span2::call_site())
                };

                if let Some(ty) = &returned_widget.ty {
                    stream.extend(quote! {
                        #name : #ty =
                    });
                } else {
                    stream.extend(quote! {
                        let #name =
                    });
                }
            }
        }
        stream
    }

    pub fn factory_expr(&self) -> Option<TokenStream2> {
        if let PropertyType::Factory(expr) = self {
            Some(expr.to_token_stream())
        } else {
            None
        }
    }
}
