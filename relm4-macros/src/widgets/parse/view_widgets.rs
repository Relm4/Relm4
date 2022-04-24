use syn::{
    parse::{Parse, ParseStream, Result},
    Error, Ident, Token,
};

use proc_macro2::Span as Span2;

use crate::widgets::{TopLevelWidget, ViewWidgets};

impl Parse for ViewWidgets {
    fn parse(input: ParseStream) -> Result<Self> {
        let span = input.span();

        let first_widget: TopLevelWidget = input.parse()?;
        let mut root_exists = first_widget.root_attr.is_some();
        let mut top_level_widgets = vec![first_widget];

        while input.peek(Token![,]) {
            let _colon: Token![,] = input.parse()?;
            let widget: TopLevelWidget = input.parse()?;
            if let Some(root_attr) = &widget.root_attr {
                if root_exists {
                    return Err(Error::new(root_attr.span(), "You cannot have two roots."));
                } else {
                    root_exists = true;
                }
            }
            top_level_widgets.push(widget);
        }

        if !root_exists && top_level_widgets.len() == 1 {
            top_level_widgets[0].root_attr = Some(Ident::new("root", Span2::call_site()));
        }

        if !input.is_empty() {
            Err(input.error("Expected end of input. Maybe a missing colon?"))
        } else {
            Ok(Self {
                span,
                top_level_widgets,
            })
        }
    }
}
