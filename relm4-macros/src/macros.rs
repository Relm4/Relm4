use proc_macro::Span;
use syn::{spanned::Spanned, Error, Macro, Result};

use crate::widgets::Widget;
use crate::args::Args;
use crate::struct_field::StructField;

pub(super) struct Macros {
    pub widgets: Widget,
    pub additional_fields: Option<Args<StructField>>,
}

impl Macros {
    pub fn new(macros: &[Macro], span: Span) -> Result<Self> {
        let mut additional_fields = None;
        let mut widgets = None;

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
                }
                if widgets.is_some() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "widget macro defined multiple times",
                    ));
                }
                widgets = Some(syn::parse_macro_input::parse::<Widget>(tokens.into())?);
            } else if ident == "additional_fields" {
                if additional_fields.is_some() {
                    return Err(Error::new(
                        mac.span().unwrap().into(),
                        "additional_fields macro defined multiple times",
                    ));
                }
                additional_fields = Some(syn::parse_macro_input::parse::<Args<StructField>>(tokens.into())?);
            } else {
                return Err(Error::new(
                    mac.span().unwrap().into(),
                    "Expected identifier view or additional_fields",
                ));
            }
        }

        Ok(Macros {
            widgets: widgets
                .ok_or_else(|| Error::new(span.into(), "No view macro in impl block"))?,
            additional_fields,
        })
    }
}
