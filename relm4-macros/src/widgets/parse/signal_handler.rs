use syn::{parse::ParseStream, Expr, Result, Token};

use crate::widgets::{Args, SignalHandler};

impl SignalHandler {
    pub(super) fn parse_with_args(input: ParseStream, args: Option<Args<Expr>>) -> Result<Self> {
        let closure = input.parse()?;

        let handler_id = if input.peek(Token![@]) {
            let _arrow: Token![@] = input.parse()?;
            input.parse()?
        } else {
            None
        };

        Ok(Self {
            closure,
            handler_id,
            args,
        })
    }
}
