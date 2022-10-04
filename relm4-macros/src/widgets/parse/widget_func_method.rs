use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Result};

use crate::widgets::WidgetFuncMethod;

impl Parse for WidgetFuncMethod {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        let turbofish = input.parse().ok();
        let args = if input.peek(token::Paren) {
            let inner_input;
            parenthesized!(inner_input in input);
            Some(Punctuated::parse_terminated(&inner_input)?)
        } else {
            None
        };

        Ok(Self {
            ident,
            turbofish,
            args,
        })
    }
}
