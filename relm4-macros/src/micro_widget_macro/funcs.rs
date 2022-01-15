use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Error, ImplItemMethod, Result};

use crate::parse_func;

pub(super) struct Funcs {
    pub pre_init: Option<TokenStream2>,
    pub post_init: Option<TokenStream2>,
    pub pre_view: Option<TokenStream2>,
    pub post_view: Option<TokenStream2>,
}

impl Funcs {
    pub fn new(funcs: &[ImplItemMethod]) -> Result<Self> {
        let mut pre_init = None;
        let mut post_init = None;
        let mut pre_view = None;
        let mut post_view = None;

        for func in funcs {
            let ident = &func.sig.ident;
            let stmts = &func.block.stmts;
            let tokens = quote! { #(#stmts)* };

            if ident == "pre_init" {
                parse_func!(pre_init, func, tokens);
            } else if ident == "position" {
                parse_func!(post_init, func, tokens);
            } else if ident == "pre_view" {
                parse_func!(pre_view, func, tokens);
            } else if ident == "post_view" {
                parse_func!(post_view, func, tokens);
            } else {
                return Err(Error::new(
                    func.span().unwrap().into(),
                    "Expected identifier `pre_init`, `post_init`, `pre_view` or `post_view`.",
                ));
            }
        }

        Ok(Funcs {
            pre_init,
            post_init,
            pre_view,
            post_view,
        })
    }
}
