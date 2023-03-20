use syn::parse::{Parse, ParseStream, Result};
use syn::{Error, Ident, Token};

use crate::widgets::{TopLevelWidget, ViewWidgets};

impl Parse for ViewWidgets {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let span = input.span();

        let first_widget = TopLevelWidget::parse(input);
        let mut root_exists = first_widget.root_attr.is_some();
        let mut top_level_widgets = vec![first_widget];

        while input.peek(Token![,]) {
            let _colon: Token![,] = input.parse()?;

            // Allow ending with a comma after the last widget
            if input.is_empty() {
                break;
            }

            let widget = TopLevelWidget::parse(input);
            if let Some(root_attr) = &widget.root_attr {
                if root_exists {
                    return Err(Error::new(root_attr.span(), "You cannot have two roots."));
                }
                root_exists = true;
            }
            top_level_widgets.push(widget);
        }

        if !root_exists && top_level_widgets.len() == 1 {
            top_level_widgets[0].root_attr = Some(Ident::new("root", input.span()));
        }

        if input.is_empty() {
            Ok(Self {
                span,
                top_level_widgets,
            })
        } else {
            Err(input.error("Expected end of input. Maybe a missing colon?"))
        }
    }
}
