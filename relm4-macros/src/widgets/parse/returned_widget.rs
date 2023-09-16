use syn::parse::{Parse, ParseStream};
use syn::{braced, Ident, Result, Token};

use crate::widgets::{parse_util, Properties, ReturnedWidget};

impl Parse for ReturnedWidget {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut is_optional = false;

        let (name, ty) = if input.peek(Ident) {
            let name = input.parse()?;

            let _colon: Token![:] = input.parse()?;
            let ty = input.parse()?;

            if input.peek(Token![?]) {
                let _mark: Token![?] = input.parse()?;
                is_optional = true;
            }

            (Some(name), Some(ty))
        } else {
            if input.peek(Token![?]) {
                let _mark: Token![?] = input.parse()?;
                is_optional = true;
            }

            (None, None)
        };

        let name = name.unwrap_or_else(|| parse_util::unique_ident_from_parts(["returned_widget"]));

        let inner;
        let _token = braced!(inner in input);
        let properties = Properties::parse(&inner);

        Ok(ReturnedWidget {
            name,
            ty,
            properties,
            is_optional,
        })
    }
}
