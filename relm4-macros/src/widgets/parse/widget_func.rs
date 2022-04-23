use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Result, Token,
};

use crate::widgets::{WidgetFunc, WidgetFuncPath};

impl WidgetFunc {
    pub(super) fn parse_with_path(input: ParseStream, path: WidgetFuncPath) -> Result<Self> {
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

        Ok(WidgetFunc { path, args, ty })
    }
}

impl Parse for WidgetFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        Self::parse_with_path(input, path)
    }
}
