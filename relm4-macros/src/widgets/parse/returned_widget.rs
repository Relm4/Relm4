use proc_macro2::Span as Span2;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, Ident, Result, Token};

use crate::widgets::ReturnedWidget;

impl Parse for ReturnedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
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

        let name = name.unwrap_or_else(|| {
            crate::util::idents_to_snake_case(
                [Ident::new("_returned_widget", Span2::call_site())].iter(),
                ty.span(),
            )
        });

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        Ok(ReturnedWidget {
            name,
            ty,
            properties,
            is_optional,
        })
    }
}
