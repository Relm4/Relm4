use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::{punctuated::Punctuated, token, token::Mut, Expr, ExprClosure, Generics, Ident, Path};

use crate::args::Args;

mod gen;
mod parse;

#[derive(Debug)]
pub(super) struct Tracker {
    bool_fn: Expr,
    update_fns: Vec<Expr>,
}

#[derive(Debug)]
pub(super) enum PropertyType {
    Expr(Expr),
    Track(Tracker),
    Parent(Expr),
    Args(Args<Expr>),
    Connect(ExprClosure),
    ConnectComponent(ExprClosure),
    Watch(TokenStream2),
    Factory(Expr),
    Widget(Widget),
}

#[derive(Debug)]
pub enum PropertyName {
    Ident(Ident),
    Path(Path),
    RelmContainerExtAssign,
}

#[derive(Debug)]
pub(super) struct Property {
    /// Either a path or just an ident
    pub name: PropertyName,
    pub ty: PropertyType,
    pub generics: Option<Generics>,
    /// Optional arguments like param_name(arg1, arg2, ...)
    pub args: Option<Args<Expr>>,
    /// Assign with an ?
    pub optional_assign: bool,
    /// Iterate through elements to generate tokens
    pub iterative: bool,
}

#[derive(Debug)]
pub(super) struct Properties {
    pub properties: Vec<Property>,
}

/// The function that intitalizes the widget.
///
/// This might be a real function or just something like `gtk::Label`.
#[derive(Debug)]
pub(super) struct WidgetFunc {
    pub path_segments: Vec<Ident>,
    pub args: Option<Punctuated<Expr, token::Comma>>,
    pub ty: Option<Vec<Ident>>,
    pub span: Span2,
}

#[derive(Debug)]
pub(super) struct Widget {
    pub mutable: Option<Mut>,
    pub name: Ident,
    pub func: WidgetFunc,
    pub properties: Properties,
    pub wrapper: Option<Ident>,
    pub ref_token: Option<token::And>,
    pub deref_token: Option<token::Star>,
    pub returned_widget: Option<ReturnedWidget>,
}

#[derive(Debug)]
pub(super) struct ReturnedWidget {
    pub name: Ident,
    pub ty: Option<Path>,
    pub properties: Properties,
    pub is_optional: bool,
}
