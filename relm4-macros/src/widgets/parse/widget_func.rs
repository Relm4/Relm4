use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, Path, Token, token};

use crate::widgets::{ParseError, WidgetFunc, parse_util};

impl WidgetFunc {
    pub(super) fn parse_with_path(input: ParseStream<'_>, path: &Path) -> Result<Self, ParseError> {
        match Self::parse_with_path_internal(input, path) {
            Ok(func) => Ok(func),
            Err(err) => Err(err.add_path(path)),
        }
    }

    fn parse_with_path_internal(input: ParseStream<'_>, path: &Path) -> Result<Self, ParseError> {
        if input.peek(Ident) {
            return Err(ParseError::Generic(
                syn::Error::new(
                    path.span()
                        .join(input.span())
                        .unwrap_or_else(|| input.span()),
                    "A path must not be followed by an identifier",
                )
                .into_compile_error(),
            )
            .add_path(path));
        }

        let args = if input.peek(token::Paren) {
            let paren_input = parse_util::parens(input)?;
            Some(paren_input.call(Punctuated::parse_terminated)?)
        } else {
            None
        };

        let method_chain = if input.peek(token::Dot) {
            let _dot: token::Dot = input.parse()?;
            Some(Punctuated::parse_separated_nonempty(input)?)
        } else {
            None
        };

        let ty = if input.peek(Token! [->]) {
            let _token: Token! [->] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(WidgetFunc {
            path: path.clone(),
            args,
            method_chain,
            ty,
        })
    }
}

impl WidgetFunc {
    pub(super) fn parse(input: ParseStream<'_>) -> Result<Self, ParseError> {
        let path = &input.parse()?;
        Self::parse_with_path(input, path)
    }
}
