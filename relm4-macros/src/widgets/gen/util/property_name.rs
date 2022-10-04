use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::Ident;

use crate::widgets::gen::PropertyName;

impl PropertyName {
    pub(crate) fn assign_fn_stream(&self, w_name: &Ident) -> TokenStream2 {
        match self {
            PropertyName::Ident(ident) => {
                quote! { #w_name.#ident }
            }
            PropertyName::Path(path) => path.to_token_stream(),
            PropertyName::RelmContainerExtAssign => {
                quote_spanned! { w_name.span() => relm4::RelmContainerExt::container_add }
            }
        }
    }

    pub(crate) fn assign_args_stream(&self, w_name: &Ident) -> Option<TokenStream2> {
        match self {
            PropertyName::Ident(_) => None,
            PropertyName::Path(_) | PropertyName::RelmContainerExtAssign => {
                Some(quote_spanned! { w_name.span() => & #w_name, })
            }
        }
    }
}
