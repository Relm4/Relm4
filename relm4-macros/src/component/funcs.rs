use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{spanned::Spanned, Error, ImplItemMethod, Result};

pub(super) struct Funcs {
    pub unhandled_fns: Vec<ImplItemMethod>,
    pub init_parts: ImplItemMethod,
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
    pub fn new(mut funcs: Vec<ImplItemMethod>) -> Result<Self> {
        let mut init_parts = None;
        let mut unhandled_fns = Vec::new();
        let mut pre_view = None;
        let mut post_view = None;

        for func in funcs.drain(..) {
            let ident = &func.sig.ident;

            if ident == "init_parts" {
                if init_parts.is_some() {
                    return Err(Error::new(
                        func.span().unwrap().into(),
                        "`init_parts` method defined multiple times",
                    ));
                } else {
                    init_parts = Some(func);
                }
            } else if ident == "pre_view" {
                let stmts = &func.block.stmts;
                let tokens = quote! { #(#stmts)* };
                parse_func!(pre_view, func, tokens);
            } else if ident == "post_view" {
                let stmts = &func.block.stmts;
                let tokens = quote! { #(#stmts)* };
                parse_func!(post_view, func, tokens);
            } else {
                unhandled_fns.push(func);
            }
        }

        let init_parts = init_parts
            .ok_or_else(|| Error::new(Span2::call_site(), "`init_parts` method isn't defined"))?;

        Ok(Funcs {
            init_parts,
            pre_view,
            post_view,
            unhandled_fns,
        })
    }
}
