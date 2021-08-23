use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Field, Result, Token,
};

pub struct AdditionalFields {
    pub inner: Punctuated<Field, Token![,]>,
}

impl Parse for AdditionalFields {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(AdditionalFields {
            inner: input.parse_terminated(Field::parse_named)?,
        })
    }
}

impl ToTokens for AdditionalFields {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(self.inner.to_token_stream());
    }
}
