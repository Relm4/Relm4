use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

use crate::widgets::gen::PropertyType;
impl PropertyType {
    pub fn return_assign_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Widget(widget) = self {
            if let Some(returned_widget) = &widget.returned_widget {
                let mut stream = TokenStream2::new();

                let name = &returned_widget.name;

                if let Some(ty) = &returned_widget.ty {
                    stream.extend(quote! {
                        let #name : #ty
                    });
                } else {
                    stream.extend(quote! {
                        let #name
                    });
                }
                Some(stream)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn factory_expr(&self) -> Option<TokenStream2> {
        if let PropertyType::Factory(expr) = self {
            Some(expr.to_token_stream())
        } else {
            None
        }
    }
}
