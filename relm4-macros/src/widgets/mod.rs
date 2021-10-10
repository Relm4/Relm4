use proc_macro2::Span as Span2;
use proc_macro2::TokenStream as TokenStream2;
use syn::{punctuated::Punctuated, spanned::Spanned, token, Expr, ExprClosure, Ident, Lit, Path, Generics};

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
    Value(Lit),
    Track(Tracker),
    Component(Expr),
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
}

impl Spanned for PropertyName {
    fn span(&self) -> Span2 {
        match self {
            PropertyName::Ident(ident) => ident.span(),
            PropertyName::Path(path) => path.span(),
        }
    }
}

#[derive(Debug)]
pub(super) struct Property {
    pub name: PropertyName,
    pub ty: PropertyType,
    pub generics: Option<Generics>,
    pub args: Option<Args<Expr>>,
    pub optional_assign: bool,
    pub iterative: bool,
}

#[derive(Debug)]
pub(super) struct Properties {
    pub properties: Vec<Property>,
}

#[derive(Debug)]
pub(super) struct WidgetFunc {
    pub path_segments: Vec<Ident>,
    pub args: Option<Punctuated<Expr, token::Comma>>,
    pub ty: Option<Vec<Ident>>,
}

impl Spanned for WidgetFunc {
    fn span(&self) -> Span2 {
        self.path_segments
            .first()
            .expect("Expected path segments in WidgetFunc")
            .span()
    }
}

#[derive(Debug)]
pub(super) struct Widget {
    pub name: Ident,
    pub func: WidgetFunc,
    pub properties: Properties,
    pub wrapper: Option<Ident>,
    pub assign_as_ref: bool,
}

impl<'a> Widget {
    pub fn get_widget_list(&'a self, widgets: &mut Vec<&'a Widget>) {
        widgets.push(self);

        for prop in &self.properties.properties {
            let ty = &prop.ty;
            if let PropertyType::Widget(widget) = ty {
                widget.get_widget_list(widgets);
            }
        }
    }
}
