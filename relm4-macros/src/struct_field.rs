use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::{Parse, ParseStream}, Field, Result};
use quote::ToTokens;

pub struct StructField {
    inner: Field,
}

impl StructField {
    pub fn ident_token(&self) -> TokenStream2 {
        self.inner.ident.to_token_stream()
    }
}

impl Parse for StructField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StructField {
            inner: input.call(Field::parse_named)?
        })
    }
}

impl ToTokens for StructField {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        self.inner.to_tokens(stream);
    }
}
