use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Pair, Punctuated};
use syn::spanned::Spanned;
use syn::{Error, Result, Token};

#[derive(Debug)]
pub(super) struct Args<T>
where
    T: Parse + ToTokens,
{
    pub(super) inner: Vec<T>,
}

impl<T> Parse for Args<T>
where
    T: Parse + ToTokens,
{
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let punct: Punctuated<T, Token![,]> = input.call(Punctuated::parse_terminated)?;
        if punct.is_empty() {
            return Err(Error::new(
                input.span(),
                "Expected at least one element. This is probably caused by empty arguments and macros.",
            ));
        }
        let inner = punct.into_pairs().map(Pair::into_value).collect();

        Ok(Args { inner })
    }
}

impl<T> ToTokens for Args<T>
where
    T: Parse + ToTokens,
{
    fn to_tokens(&self, out: &mut TokenStream2) {
        let mut iter = self.inner.iter();

        let first = iter.next().unwrap();
        out.extend(quote_spanned! {
            first.span() => #first
        });

        for expr in iter {
            out.extend(quote_spanned! {
                expr.span() => ,#expr
            });
        }
    }
}
