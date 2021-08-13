/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

#[derive(Debug)]
pub struct Attrs {
    pub_vis: Option<Token![pub]>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let pub_vis = if input.is_empty() {
            None
        } else {
            Some(input.parse()?)
        };

        Ok(Attrs { pub_vis })
    }
}

impl ToTokens for Attrs {
    fn to_tokens(&self, out: &mut TokenStream2) {
        if let Some(vis_pub) = &self.pub_vis {
            out.extend(quote! { #vis_pub });
        }
    }
}
