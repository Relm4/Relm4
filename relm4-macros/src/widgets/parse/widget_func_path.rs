use syn::{
    parse::{Parse, ParseStream},
    token, Result,
};

use crate::widgets::WidgetFuncPath;

impl Parse for WidgetFuncPath {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek2(token::Dot) {
            Self::Method(input.parse()?)
        } else {
            Self::Path(input.parse()?)
        })
    }
}
