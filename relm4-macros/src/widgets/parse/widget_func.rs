use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, Result, Token,
};

use crate::widgets::WidgetFunc;

impl Parse for WidgetFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut path_segments = Vec::new();
        let mut args = None;
        let mut ty = None;

        let first_segment: Ident = input.parse()?;
        let span = first_segment.span();
        path_segments.push(first_segment);

        loop {
            if input.peek(Ident) {
                path_segments.push(input.parse()?);
            } else if input.peek(Token! [::]) {
                let _colon: Token![::] = input.parse()?;
            } else if input.peek(token::Paren) {
                let paren_input;
                parenthesized!(paren_input in input);
                args = Some(paren_input.call(Punctuated::parse_terminated)?);
                if input.peek(Token! [->]) {
                    let _token: Token! [->] = input.parse()?;
                    let mut ty_path = vec![input.parse()?];

                    loop {
                        if input.peek(Ident) {
                            ty_path.push(input.parse()?);
                        } else if input.peek(Token! [::]) {
                            let _colon: Token![::] = input.parse()?;
                        } else {
                            break;
                        }
                    }
                    ty = Some(ty_path);
                }
                break;
            } else {
                break;
            }
        }

        Ok(WidgetFunc {
            path_segments,
            args,
            ty,
            span,
        })
    }
}
