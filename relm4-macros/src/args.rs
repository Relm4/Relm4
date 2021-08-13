/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, Token,
};

#[derive(Debug)]
pub struct Args<T>
where
    T: Parse + ToTokens,
{
    pub inner: Vec<T>,
}

impl<T> Parse for Args<T>
where
    T: Parse + ToTokens,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let punct: Punctuated<T, Token![,]> = input.call(Punctuated::parse_separated_nonempty)?;
        let inner = punct.into_pairs().map(|pair| pair.into_value()).collect();

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
