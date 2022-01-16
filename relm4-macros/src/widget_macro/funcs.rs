use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Error, ImplItemMethod, Result};

pub(super) struct Funcs {
    pub pre_init: Option<TokenStream2>,
    pub post_init: Option<TokenStream2>,
    pub pre_connect_parent: Option<TokenStream2>,
    pub post_connect_parent: Option<TokenStream2>,
    pub pre_view: Option<TokenStream2>,
    pub post_view: Option<TokenStream2>,
}

macro_rules! parse_func {
    ($name:ident, $func:ident, $tokens:ident) => {
        if $name.is_some() {
            return Err(Error::new(
                $func.span().unwrap().into(),
                &format!("{} method defined multiple times", stringify!($name)),
            ));
        }
        $name = Some($tokens);
    };
}

impl Funcs {
    pub fn new(funcs: &[ImplItemMethod]) -> Result<Self> {
        let mut pre_init = None;
        let mut post_init = None;
        let mut pre_connect_parent = None;
        let mut post_connect_parent = None;
        let mut pre_view = None;
        let mut post_view = None;

        for func in funcs {
            let ident = &func.sig.ident;
            let stmts = &func.block.stmts;
            let tokens = quote! { #(#stmts)* };

            if ident == "pre_init" {
                parse_func!(pre_init, func, tokens);
            } else if ident == "post_init" {
                parse_func!(post_init, func, tokens);
            } else if ident == "pre_connect_parent" {
                parse_func!(pre_connect_parent, func, tokens);
            } else if ident == "post_connect_parent" {
                parse_func!(post_connect_parent, func, tokens);
            } else if ident == "pre_view" {
                parse_func!(pre_view, func, tokens);
            } else if ident == "post_view" {
                parse_func!(post_view, func, tokens);
            } else {
                return Err(Error::new(
                    func.span().unwrap().into(),
                    "Expected identifier `pre_init`, `post_init`, `pre_connect_parent`, `post_connect_parent`, `pre_view` or `post_view`.",
                ));
            }
        }

        Ok(Funcs {
            pre_init,
            post_init,
            pre_connect_parent,
            post_connect_parent,
            pre_view,
            post_view,
        })
    }
}
