use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Path, Result, Token};

use crate::widgets::WidgetFunc;

impl WidgetFunc {
    pub(super) fn parse_with_path(input: ParseStream, path: Path) -> Result<Self> {
        let mut args = None;
        let mut ty = None;

        if input.peek(token::Paren) {
            let paren_input;
            parenthesized!(paren_input in input);
            args = Some(paren_input.call(Punctuated::parse_terminated)?);
            if input.peek(Token! [->]) {
                let _token: Token! [->] = input.parse()?;
                let ty_path = input.parse()?;
                ty = Some(ty_path);
            }
        } else if input.peek(Token! [->]) {
            let _token: Token! [->] = input.parse()?;
            let ty_path = input.parse()?;
            ty = Some(ty_path);
        }

        let method_chain = if input.peek(token::Dot) {
            let _dot: token::Dot = input.parse()?;
            Some(Punctuated::parse_separated_nonempty(input)?)
        } else {
            None
        };

        Ok(WidgetFunc {
            path,
            args,
            method_chain,
            ty,
        })
    }
}

impl Parse for WidgetFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        Self::parse_with_path(input, path)
    }
}
