use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, Error, Result,
};

use crate::widgets::{PropertyFunc, WidgetFunc, WidgetFuncPath};

impl Parse for PropertyFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: WidgetFuncPath = input.parse()?;

        if input.peek(token::Paren) {
            Ok(Self::Func(WidgetFunc::parse_with_path(input, path)?))
        } else {
            match path {
                WidgetFuncPath::Path(path) => Ok(if let Some(ident) = path.get_ident() {
                    Self::Ident(ident.clone())
                } else {
                    Self::Path(path)
                }),
                WidgetFuncPath::Method(method) => {
                    Err(Error::new(method.path.span(), "Expected a method call"))
                }
            }
        }
    }
}
