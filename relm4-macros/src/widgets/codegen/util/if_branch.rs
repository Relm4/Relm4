use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::{IfBranch, IfCondition};

impl IfBranch {
    pub(crate) fn update_stream(
        &self,
        stream: &mut TokenStream2,
        inner_update_tokens: &TokenStream2,
        index: usize,
    ) {
        let index = index.to_string();
        stream.extend(match &self.cond {
            IfCondition::If(if_token, expr) => quote! {
                #if_token #expr
            },
            IfCondition::ElseIf(else_token, if_token, expr) => quote! {
                #else_token #if_token #expr
            },
            IfCondition::Else(else_token) => quote! {
                #else_token
            },
        });
        stream.extend(quote! {
            {
                let page_active: bool = (current_page == #index);
                #inner_update_tokens
                #index
            }
        });
    }
}
