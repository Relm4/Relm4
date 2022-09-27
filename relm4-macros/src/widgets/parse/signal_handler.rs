use syn::parse::ParseStream;
use syn::{Expr, Result, Token};

use crate::widgets::{Args, ClosureSignalHandler, SignalHandler, SignalHandlerVariant};

impl SignalHandler {
    pub(super) fn parse_with_args(
        input: ParseStream<'_>,
        args: Option<Args<Expr>>,
    ) -> Result<Self> {
        let inner = if args.is_some() || input.peek(Token![move]) || input.peek(Token![|]) {
            ClosureSignalHandler::parse_with_args(input, args).map(SignalHandlerVariant::Closure)
        } else {
            input.parse().map(SignalHandlerVariant::Expr)
        }?;

        let handler_id = if input.peek(Token![@]) {
            let _arrow: Token![@] = input.parse()?;
            input.parse()?
        } else {
            None
        };

        Ok(Self { inner, handler_id })
    }
}

impl ClosureSignalHandler {
    pub(super) fn parse_with_args(
        input: ParseStream<'_>,
        args: Option<Args<Expr>>,
    ) -> Result<Self> {
        let closure = input.parse()?;

        Ok(Self { closure, args })
    }
}
