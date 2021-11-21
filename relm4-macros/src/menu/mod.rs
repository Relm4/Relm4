use syn::{punctuated::Punctuated, token::Comma, Expr, Ident, LitStr, Path};

mod gen;
mod parse;

#[derive(Debug)]
pub struct Menus {
    items: Punctuated<Menu, Comma>,
}

#[derive(Debug)]
struct Menu {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}

#[derive(Debug)]
enum MenuItem {
    Entry(MenuEntry),
    Section(MenuSection),
}

#[derive(Debug)]
struct MenuEntry {
    string: LitStr,
    action_ty: Path,
    value: Option<Expr>,
}

#[derive(Debug)]
struct MenuSection {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}
