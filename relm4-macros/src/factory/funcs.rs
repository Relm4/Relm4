use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Ident, ImplItemMethod, Result};

use crate::util;

pub(super) struct Funcs {
    pub unhandled_fns: Vec<ImplItemMethod>,
    pub init_widgets: Option<ImplItemMethod>,
    pub pre_view: Option<TokenStream2>,
    pub post_view: Option<TokenStream2>,
    pub root_name: Ident,
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
        let mut init_widgets = None;
        let mut unhandled_fns = Vec::new();
        let mut pre_view = None;
        let mut post_view = None;
        let mut root_name = None;

        for func in funcs.drain(..) {
            let ident = &func.sig.ident;

            if ident == "init_widgets" {
                if init_widgets.is_some() {
                    return Err(Error::new(
                        func.span().unwrap().into(),
                        "`init_widgets` method defined multiple times",
                    ));
                } else {
                    root_name = Some(util::get_ident_of_nth_func_input(&func, 2)?);
                    init_widgets = Some(func);
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

        let root_name = root_name.unwrap_or_else(|| Ident::new("root", Span2::call_site()));

        Ok(Funcs {
            init_widgets,
            pre_view,
            post_view,
            unhandled_fns,
            root_name,
        })
    }
}
