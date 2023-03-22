use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};

use crate::widgets::IfCondition;

impl Parse for IfCondition {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![if]) {
            Ok(Self::If(input.parse()?, input.parse()?))
        } else if input.peek(Token![else]) {
            let else_token = input.parse()?;
            if input.peek(Token![if]) {
                Ok(Self::ElseIf(else_token, input.parse()?, input.parse()?))
            } else {
                Ok(Self::Else(else_token))
            }
        } else {
            Err(input.error("Expected `if`, `if else` or `else`"))
        }
    }
}
