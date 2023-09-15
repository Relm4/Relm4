use proc_macro2::Span as Span2;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Ident, Path, Result, Token};

use crate::menu::SubMenu;

use super::{Menu, MenuElement, MenuEntry, MenuItem, MenuSection, Menus};

syn::custom_keyword!(custom);

impl Parse for Menus {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let items = input.call(Punctuated::parse_separated_nonempty)?;

        Ok(Menus { items })
    }
}

impl Parse for Menu {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;
        let _colon: Token![:] = input.parse()?;

        let braced_input;
        braced!(braced_input in input);

        let items = braced_input.call(Punctuated::parse_terminated)?;

        Ok(Menu { name, items })
    }
}

impl Parse for MenuItem {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let expr = input.parse()?;

        Ok(if input.peek(Token![=>]) {
            let _arrow: Token![=>] = input.parse()?;
            let action_ty = input.call(Path::parse_mod_style)?;

            let value = if input.peek(token::Paren) {
                let paren_input;
                parenthesized!(paren_input in input);
                Some(paren_input.parse()?)
            } else {
                None
            };

            Self::Entry(Box::new(MenuEntry {
                expr,
                action_ty,
                value,
            }))
        } else {
            let braced_input;
            braced!(braced_input in input);

            let items = braced_input.call(Punctuated::parse_terminated)?;

            Self::SubMenu(Box::new(SubMenu { expr, items }))
        })
    }
}

impl Parse for MenuElement {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(if input.peek(custom) {
            let _custom: custom = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            input.parse().map(MenuElement::Custom)?
        } else if input.peek2(Token![!]) {
            input.parse().map(MenuElement::Section)?
        } else {
            input.parse().map(MenuElement::Item)?
        })
    }
}

impl Parse for MenuSection {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        assert!(name == "section");
        let _excl: Token![!] = input.parse()?;

        let braced_input;
        braced!(braced_input in input);

        let items = braced_input.call(Punctuated::parse_terminated)?;
        let name = section_name();

        Ok(MenuSection { name, items })
    }
}

fn section_name() -> Ident {
    use std::sync::atomic::{AtomicU8, Ordering};
    static COUNTER: AtomicU8 = AtomicU8::new(0);

    let value = COUNTER.fetch_add(1, Ordering::Relaxed);

    Ident::new(&format!("_section_{value}"), Span2::mixed_site())
}
