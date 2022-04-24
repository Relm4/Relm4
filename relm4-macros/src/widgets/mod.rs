use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::{
    punctuated::Punctuated, token, token::Mut, Expr, ExprClosure, Ident, MethodTurbofish, Path,
};

use crate::args::Args;

mod gen;
mod parse;
mod span;
mod util;

pub(super) struct ViewWidgets {
    pub(super) span: Span2,
    pub(super) top_level_widgets: Vec<TopLevelWidget>,
}

pub(super) struct TopLevelWidget {
    pub(super) root_attr: Option<Ident>,
    pub(super) inner: Widget,
}

pub(super) enum PropertyType {
    Assign(AssignProperty),
    SignalHandler(SignalHandler),
    Widget(Widget),
}

pub(super) enum AssignPropertyAttr {
    None,
    Watch,
    Track(TokenStream2),
}

pub(super) struct AssignProperty {
    attr: AssignPropertyAttr,
    /// Optional arguments like param_name[arg1, arg2, ...]
    args: Option<Args<Expr>>,
    expr: Expr,
    /// Assign with an ?
    optional_assign: bool,
    /// Iterate through elements to generate tokens
    iterative: bool,
}

pub(super) struct SignalHandler {
    closure: ExprClosure,
    handler_id: Option<Ident>,
    args: Option<Args<Expr>>,
}

pub(super) enum PropertyFunc {
    Ident(Ident),
    Path(Path),
    Func(WidgetFunc),
}

pub enum PropertyName {
    Ident(Ident),
    Path(Path),
    RelmContainerExtAssign,
}

pub(super) struct Property {
    /// Either a path or just an ident
    name: PropertyName,
    pub(super) ty: PropertyType,
}

pub(super) struct Properties {
    pub(super) properties: Vec<Property>,
}

/// The function that initializes the widget.
///
/// This might be a real function or just something like `gtk::Label`.
pub(super) struct WidgetFunc {
    path: WidgetFuncPath,
    args: Option<Punctuated<Expr, token::Comma>>,
    ty: Option<Path>,
}

pub(super) struct WidgetMethodCall {
    path: Punctuated<Ident, token::Dot>,
    turbofish: Option<MethodTurbofish>,
}

pub(super) enum WidgetFuncPath {
    Path(Path),
    Method(WidgetMethodCall),
}

pub(super) struct Widget {
    doc_attr: Option<TokenStream2>,
    attr: WidgetAttr,
    mutable: Option<Mut>,
    pub(super) name: Ident,
    pub(super) func: WidgetFunc,
    args: Option<Args<Expr>>,
    pub(super) properties: Properties,
    wrapper: Option<Ident>,
    ref_token: Option<token::And>,
    deref_token: Option<token::Star>,
    pub(super) returned_widget: Option<ReturnedWidget>,
}

#[derive(PartialEq)]
pub(super) enum WidgetAttr {
    None,
    Local,
    LocalRef,
}

pub(super) struct ReturnedWidget {
    pub(super) name: Ident,
    ty: Option<Path>,
    pub(super) properties: Properties,
    is_optional: bool,
}

pub(super) enum Attr {
    Doc(TokenStream2),
    Local(Ident),
    LocalRef(Ident),
    Root(Ident),
    Iterate(Ident),
    Watch(Ident),
    Track(Ident, Option<Box<Expr>>),
}

pub(super) struct Attrs {
    inner: Vec<Attr>,
}
