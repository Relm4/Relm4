use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, Ident, LitStr, Path};

mod gen;
mod parse;

#[derive(Debug)]
pub(crate) struct Menus {
    items: Punctuated<Menu, Comma>,
}

#[derive(Debug)]
struct Menu {
    name: Ident,
    items: Punctuated<MenuElement, Comma>,
}

#[derive(Debug)]
enum MenuElement {
    Item(Box<MenuItem>),
    Custom(LitStr),
    Section(MenuSection),
}

#[derive(Debug)]
enum MenuItem {
    Entry(MenuEntry),
    SubMenu(SubMenu)
}

#[derive(Debug)]
struct MenuEntry {
    expr: Expr,
    action_ty: Path,
    value: Option<Expr>,
}

#[derive(Debug)]
struct SubMenu {
    expr: Expr,
    items: Punctuated<MenuElement, Comma>,
}

#[derive(Debug)]
struct MenuSection {
    name: Ident,
    items: Punctuated<MenuElement, Comma>,
}
