use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

use crate::widgets::WidgetFuncPath;

impl ToTokens for WidgetFuncPath {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            WidgetFuncPath::Path(path) => path.to_tokens(tokens),
            WidgetFuncPath::Method(method) => method.to_tokens(tokens),
        }
    }
}
