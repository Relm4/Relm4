use proc_macro::Span;
use proc_macro2::TokenStream as TokenStream2;
use syn::{spanned::Spanned, Error, ImplItem, Result};

use crate::widgets::Widget;

pub(super) struct Macros {
    pub widgets: Widget,
    pub manual_init: Option<TokenStream2>,
    pub manual_view: Option<TokenStream2>,
}

impl Macros {
    pub fn new(span: Span, items: &[ImplItem]) -> Result<Self> {
        let mut manual_init = None;
        let mut manual_view = None;
        let mut widgets = None;

        for item in items {
            if let ImplItem::Macro(mac) = item {
                let ident = &mac
                    .mac
                    .path
                    .segments
                    .first()
                    .expect("No path segments in macro path")
                    .ident;

                if ident == "view" {
                    let tokens = mac.mac.tokens.clone();
                    if tokens.is_empty() {
                        return Err(Error::new(
                            item.span().unwrap().into(),
                            "widget macro is empty",
                        ));
                    }
                    if widgets.is_some() {
                        return Err(Error::new(
                            item.span().unwrap().into(),
                            "widget macro defined multiple times",
                        ));
                    }
                    widgets = Some(syn::parse_macro_input::parse::<Widget>(tokens.into())?);
                } else if ident == "manual_init" {
                    if manual_init.is_some() {
                        return Err(Error::new(
                            item.span().unwrap().into(),
                            "manual_init macro defined multiple times",
                        ));
                    }
                    manual_init = Some(mac.mac.tokens.clone());
                } else if ident == "manual_view" {
                    if manual_view.is_some() {
                        return Err(Error::new(
                            item.span().unwrap().into(),
                            "manual_view macro defined multiple times",
                        ));
                    }
                    manual_view = Some(mac.mac.tokens.clone());
                } else {
                    return Err(Error::new(
                        item.span().unwrap().into(),
                        "Expected identifier view, manual_init or manual_view",
                    ));
                }
            }
        }

        Ok(Macros {
            widgets: widgets
                .ok_or_else(|| Error::new(span.into(), "No view macro in impl block"))?,
            manual_init,
            manual_view,
        })
    }
}
