use syn::parse::{Parse, ParseStream};
use syn::{Result, Token};

use crate::widgets::PropertyName;

impl Parse for PropertyName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(Token![::]) || input.peek2(Token! [::]) {
            PropertyName::Path(input.parse()?)
        } else {
            PropertyName::Ident(input.parse()?)
        })
    }
}
