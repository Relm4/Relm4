use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Field, Result, Token};

#[derive(Debug)]
pub(super) struct AdditionalFields {
    pub(super) inner: Punctuated<Field, Token![,]>,
}

impl Parse for AdditionalFields {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(AdditionalFields {
            inner: input.parse_terminated(Field::parse_named, Token![,])?,
        })
    }
}

impl ToTokens for AdditionalFields {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(self.inner.to_token_stream());
    }
}
