use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, Ident, LitStr, Path};

mod gen;
mod parse;

pub struct Menus {
    items: Punctuated<Menu, Comma>,
}

struct Menu {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}

enum MenuItem {
    Entry(Box<MenuEntry>),
    Section(MenuSection),
}

struct MenuEntry {
    string: LitStr,
    action_ty: Path,
    value: Option<Expr>,
}

struct MenuSection {
    name: Ident,
    items: Punctuated<MenuItem, Comma>,
}
