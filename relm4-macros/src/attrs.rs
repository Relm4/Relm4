use proc_macro2::Delimiter;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::Ident;
use syn::Result;
use syn::Token;
use syn::Visibility;
use syn::buffer::Cursor;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::token;

#[derive(Debug)]
pub struct WidgetVisibility {
    pub_vis: Option<Visibility>,
}

impl WidgetVisibility {
    fn new() -> Self {
        WidgetVisibility{
            pub_vis: None,
        }
    }
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
    visibility_set: bool,
}

impl Attrs {
    fn new() -> Self {
        Attrs {
            visibility: WidgetVisibility::new(),
            visibility_set: false,
        }
    }

    fn stream_peek(c: Cursor) {
        if !c.eof() {
            //group it is not
            if let Some( (ident, cursor) ) = c.ident() {
                eprintln!("\t ident: {}", ident);
                return Attrs::stream_peek(cursor);
            }

            if let Some( (punct, cursor) ) = c.punct() {
                eprintln!("\t punct: {}", punct);
                return Attrs::stream_peek(cursor);
            }

            if let Some( (literal, cursor) ) = c.literal() {
                eprintln!("\t literal: {}", literal);
                return Attrs::stream_peek(cursor);
            }

            if let Some( (lifetime, cursor) ) = c.lifetime() {
                eprintln!("\t lifetime: {}", lifetime);
                return Attrs::stream_peek(cursor);
            }

            if let Some( (tree, cursor) ) = c.token_tree() {
                eprintln!("\t tree: {}", tree);
                Attrs::stream_peek(cursor);
            }
        }
    }
}

impl Parse for Attrs {
    /// Rules for parsing attributes
    /// 
    /// 1. It's fine if visibility is used unnamed so `#[widget(pub)]` must be valid
    /// 2. Widget visibility might be named `#[widget(visibility = pub)]`
    ///
    fn parse(input: ParseStream) -> Result<Self> {
        eprintln!("Input:");
        eprintln!("\tis empty: {}", input.is_empty());

        let mut attrs = Attrs::new();

        Attrs::stream_peek(input.cursor());

        if input.peek(Token![pub]) {
            let pub_vis = if input.is_empty() {
                None
            } else {
                Some(input.parse()?)
            };
            attrs.visibility.pub_vis = pub_vis;
        }
        else if input.peek(Ident) && input.peek2(Token![=]){
            let ident: Ident = input.parse()?;
            let eq: Token![=] = input.parse()?;

            let pub_vis = if input.is_empty() {
                None
            } else {
                Some(input.parse()?)
            };
            attrs.visibility.pub_vis = pub_vis;
        }

        Ok(attrs)
    }

    
}

