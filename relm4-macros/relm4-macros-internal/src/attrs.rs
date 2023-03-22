use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Async;
use syn::{Error, Result, Token, Visibility};

pub struct Attrs {
    /// Keeps information about visibility of the widget
    pub visibility: Option<Visibility>,
    /// Whether an async trait is used or not
    pub asyncness: Option<Async>,
}

pub struct SyncOnlyAttrs {
    /// Keeps information about visibility of the widget
    pub visibility: Option<Visibility>,
}

impl Parse for SyncOnlyAttrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let Attrs {
            visibility,
            asyncness,
        } = input.parse()?;

        if let Some(async_token) = asyncness {
            Err(syn::Error::new(
                async_token.span,
                "this macro doesn't support async traits",
            ))
        } else {
            Ok(Self { visibility })
        }
    }
}

impl Parse for Attrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut attrs = Attrs {
            visibility: None,
            asyncness: None,
        };

        while !input.is_empty() {
            if input.peek(Async) {
                let new_asyncness: Async = input.parse()?;
                if attrs.asyncness.is_some() {
                    return Err(syn::Error::new(
                        new_asyncness.span,
                        "cannot specify asyncness twice",
                    ));
                } else {
                    attrs.asyncness = Some(new_asyncness);
                }
            } else {
                let new_vis: Visibility = input.parse()?;
                if attrs.visibility.is_some() {
                    return Err(syn::Error::new(
                        new_vis.span(),
                        "cannot specify visibility twice",
                    ));
                } else {
                    attrs.visibility = Some(new_vis);
                }
            }

            if input.peek(Token![,]) {
                let comma: Token![,] = input.parse()?;
                if input.is_empty() {
                    // We've just consumed last token in stream (which is comma) and that's wrong
                    return Err(Error::new(comma.span, "expected visibility or `async`"));
                }
            }
        }

        Ok(attrs)
    }
}
