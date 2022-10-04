use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{token, Ident, Path, Token};

use crate::widgets::{parse_util, ParseError, WidgetFunc};

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

        let mut args = None;
        let mut ty = None;
        let mut method_chain = None;

        if input.peek(token::Paren) {
            let paren_input = parse_util::parens(input)?;
            args = Some(paren_input.call(Punctuated::parse_terminated)?);

            if input.peek(token::Dot) {
                let _dot: token::Dot = input.parse()?;
                method_chain = Some(Punctuated::parse_separated_nonempty(input)?);
            }
        }

        if input.peek(Token! [->]) {
            let _token: Token! [->] = input.parse()?;
            let ty_path = input.parse()?;
            ty = Some(ty_path);
        }

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
