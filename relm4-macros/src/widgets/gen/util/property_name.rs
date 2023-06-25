use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::Ident;

use crate::widgets::gen::{assign::AssignInfo, PropertyName};

impl PropertyName {
    pub(crate) fn assign_fn_stream(&self, info: &mut AssignInfo<'_>) -> TokenStream2 {
        let AssignInfo {
            widget_name,
            template_name,
            ..
        } = info;
        let widget_name = if let Some(template_name) = template_name {
            quote! { #template_name.#widget_name }
        } else {
            quote! { #widget_name }
        };

        match self {
            PropertyName::Ident(ident) => {
                quote! { #widget_name.#ident }
            }
            PropertyName::Path(path) => path.to_token_stream(),
            PropertyName::RelmContainerExtAssign(span) => {
                quote_spanned! { *span => #widget_name.container_add }
            }
        }
    }

    pub(crate) fn assign_args_stream(&self, w_name: &Ident) -> Option<TokenStream2> {
        match self {
            PropertyName::RelmContainerExtAssign(_) | PropertyName::Ident(_) => None,
            PropertyName::Path(_) => Some(quote_spanned! { w_name.span() => & #w_name, }),
        }
    }
}
