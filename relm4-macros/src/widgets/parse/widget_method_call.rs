use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result,
};

use crate::widgets::WidgetMethodCall;

impl Parse for WidgetMethodCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = Punctuated::parse_separated_nonempty(input)?;
        let turbofish = input.parse().ok();

        Ok(Self { path, turbofish })
    }
}
