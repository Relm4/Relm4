use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Result, Visibility,
};

#[derive(Debug)]
pub struct WidgetVisibility {
    pub_vis: Option<Visibility>,
}

impl ToTokens for WidgetVisibility {
    fn to_tokens(&self, out: &mut TokenStream2) {
        if let Some(vis_pub) = &self.pub_vis {
            vis_pub.to_tokens(out);
        }
    }
}

pub struct Attrs {
    pub visibility: WidgetVisibility,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let pub_vis = if input.is_empty() {
            None
        } else {
            Some(input.parse()?)
        };

        let visibility = WidgetVisibility{
            pub_vis,
        };

        Ok(Attrs { visibility })
    }
}

