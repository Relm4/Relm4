use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Expr, Ident, ImplItemMethod, Result, Stmt};

use crate::util;

pub(super) struct Funcs {
    pub unhandled_fns: Vec<ImplItemMethod>,
    pub init: ImplItemMethod,
    pub pre_view: Option<TokenStream2>,
    pub post_view: Option<TokenStream2>,
    pub root_name: Ident,
    pub model_name: Ident,
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
        let mut init = None;
        let mut unhandled_fns = Vec::new();
        let mut pre_view = None;
        let mut post_view = None;
        let mut root_name = None;
        let mut model_name = None;

        for func in funcs.drain(..) {
            let ident = &func.sig.ident;

            if ident == "init" {
                if init.is_some() {
                    return Err(Error::new(
                        func.span().unwrap().into(),
                        "`init` method defined multiple times",
                    ));
                } else {
                    root_name = Some(util::get_ident_of_nth_func_input(&func, 1)?);
                    match func.block.stmts.last() {
                        Some(stmt) => {
                            if let Stmt::Expr(Expr::Struct(strct)) = stmt {
                                if strct.path.segments.last().unwrap().ident == "ComponentParts" {
                                    if let Expr::Path(path) = &strct.fields.first().unwrap().expr {
                                        model_name = Some(path.path.get_ident().unwrap().clone());
                                    }
                                }
                            }
                        }
                        None => {
                            return Err(Error::new(
                                func.span().unwrap().into(),
                                "`init` method must not be empty",
                            ));
                        }
                    }
                    init = Some(func);
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

        let init =
            init.ok_or_else(|| Error::new(Span2::call_site(), "`init` method isn't defined"))?;
        // Must exist when init exists
        let root_name = root_name.unwrap();

        let model_name = model_name.unwrap_or_else(|| Ident::new("model", Span2::call_site()));

        Ok(Funcs {
            init,
            pre_view,
            post_view,
            unhandled_fns,
            root_name,
            model_name,
        })
    }
}
