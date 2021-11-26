use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

use crate::widgets::gen::PropertyType;
impl PropertyType {
    pub fn factory_expr(&self) -> Option<TokenStream2> {
        if let PropertyType::Factory(expr) = self {
            Some(expr.to_token_stream())
        } else {
            None
        }
    }
}
