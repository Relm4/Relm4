use proc_macro::Span;
use syn::{spanned::Spanned, Error, Macro, Result};

use crate::additional_fields::AdditionalFields;
use crate::menu::Menus;
use crate::widgets::ViewWidgets;

pub(super) struct Macros {
    pub view_widgets: ViewWidgets,
    pub additional_fields: Option<AdditionalFields>,
    pub menus: Option<Menus>,
}

impl Macros {
    pub fn new(macros: &[Macro], span: Span) -> Result<Self> {
        let mut additional_fields = None;
        let mut view_widgets = None;
        let mut menus = None;

        for mac in macros {
            let ident = &mac
                .path
                .segments
                .first()
                .expect("No path segments in macro path")
                .ident;
            let tokens = mac.tokens.clone();

            if ident == "view" {
                if tokens.is_empty() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "widget macro is empty",
                    ));
                } else if view_widgets.is_some() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "widget macro defined multiple times",
                    ));
                }
                view_widgets = Some(syn::parse_macro_input::parse(tokens.into())?);
            } else if ident == "additional_fields" {
                if additional_fields.is_some() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "additional_fields macro defined multiple times",
                    ));
                }
                additional_fields = Some(syn::parse_macro_input::parse::<AdditionalFields>(
                    tokens.into(),
                )?);
            } else if ident == "menu" {
                if tokens.is_empty() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "menu macro is empty",
                    ));
                } else if menus.is_some() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "menu macro defined multiple times",
                    ));
                }
                menus = Some(syn::parse_macro_input::parse::<Menus>(tokens.into())?);
            } else {
                return Err(Error::new(
                    mac.span().unwrap().into(),
                    "Expected identifier view, menu or additional_fields",
                ));
            }
        }

        Ok(Macros {
            view_widgets: view_widgets
                .ok_or_else(|| Error::new(span.into(), "No view macro in impl block"))?,
            additional_fields,
            menus,
        })
    }
}
