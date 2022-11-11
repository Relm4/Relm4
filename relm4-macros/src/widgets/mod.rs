use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::punctuated::Punctuated;
use syn::token::{Else, FatArrow, If, Match, Mut};
use syn::{token, Expr, ExprClosure, Ident, MethodTurbofish, Pat, Path, Type};

use crate::args::Args;

mod gen;
mod parse;
mod parse_util;
mod span;

#[derive(Debug)]
pub(super) struct ViewWidgets {
    pub(super) span: Span2,
    pub(super) top_level_widgets: Vec<TopLevelWidget>,
}

#[derive(Debug)]
pub(super) struct TopLevelWidget {
    pub(super) root_attr: Option<Ident>,
    pub(super) inner: Widget,
}

#[derive(Debug)]
enum PropertyType {
    Assign(AssignProperty),
    SignalHandler(SignalHandler),
    Widget(Widget),
    ConditionalWidget(ConditionalWidget),
    ParseError(ParseError),
}

#[derive(Debug)]
enum ParseError {
    Ident((Ident, TokenStream2)),
    Path((Path, TokenStream2)),
    Generic(TokenStream2),
}

#[derive(Debug)]
enum AssignPropertyAttr {
    None,
    Watch,
    /// The bool indicated whether the model type needs to
    /// be pasted in front of the track expression.
    Track((TokenStream2, bool)),
}

#[derive(Debug)]
struct AssignProperty {
    attr: AssignPropertyAttr,
    /// Optional arguments like param_name[arg1, arg2, ...]
    args: Option<Args<Expr>>,
    expr: Expr,
    /// Assign with an ?
    optional_assign: bool,
    /// Iterate through elements to generate tokens
    iterative: bool,
    block_signals: Vec<Ident>,
    chain: Option<Box<Expr>>,
}

#[derive(Debug)]
struct SignalHandler {
    inner: SignalHandlerVariant,
    handler_id: Option<Ident>,
}

#[derive(Debug)]
enum SignalHandlerVariant {
    Expr(Expr),
    Closure(ClosureSignalHandler),
}

#[derive(Debug)]
struct ClosureSignalHandler {
    closure: ExprClosure,
    args: Option<Args<Expr>>,
}

#[derive(Debug)]
enum PropertyName {
    Ident(Ident),
    Path(Path),
    RelmContainerExtAssign(Span2),
}

#[derive(Debug)]
struct Property {
    /// Either a path or just an ident
    name: PropertyName,
    ty: PropertyType,
}

#[derive(Debug, Default)]
struct Properties {
    properties: Vec<Property>,
}

/// The function that initializes the widget.
///
/// This might be a real function or just something like `gtk::Label`.
#[derive(Debug)]
struct WidgetFunc {
    path: Path,
    args: Option<Punctuated<Expr, token::Comma>>,
    method_chain: Option<Punctuated<WidgetFuncMethod, token::Dot>>,
    ty: Option<Box<Type>>,
}

#[derive(Debug)]
struct WidgetFuncMethod {
    ident: Ident,
    turbofish: Option<MethodTurbofish>,
    args: Option<Punctuated<Expr, token::Comma>>,
}

#[derive(Debug)]
pub(super) struct Widget {
    doc_attr: Option<TokenStream2>,
    attr: WidgetAttr,
    template_attr: WidgetTemplateAttr,
    mutable: Option<Mut>,
    pub(super) name: Ident,
    name_assigned_by_user: bool,
    func: WidgetFunc,
    args: Option<Args<Expr>>,
    properties: Properties,
    assign_wrapper: Option<Path>,
    ref_token: Option<token::And>,
    deref_token: Option<token::Star>,
    returned_widget: Option<ReturnedWidget>,
}

#[derive(Debug, PartialEq)]
enum WidgetAttr {
    None,
    Local,
    LocalRef,
}

#[derive(Debug, PartialEq)]
enum WidgetTemplateAttr {
    None,
    Template,
    TemplateChild,
}

#[derive(Debug)]
struct ReturnedWidget {
    name: Ident,
    ty: Option<Path>,
    properties: Properties,
    is_optional: bool,
}

#[derive(Debug)]
struct ConditionalWidget {
    doc_attr: Option<TokenStream2>,
    transition: Option<Ident>,
    assign_wrapper: Option<Path>,
    name: Ident,
    args: Option<Args<Expr>>,
    branches: ConditionalBranches,
}

#[derive(Debug)]
enum ConditionalBranches {
    If(Vec<IfBranch>),
    Match((Match, Box<Expr>, Vec<MatchArm>)),
}

#[derive(Debug)]
enum IfCondition {
    If(If, Expr),
    ElseIf(Else, If, Expr),
    Else(Else),
}

#[derive(Debug)]
struct IfBranch {
    cond: IfCondition,
    widget: Widget,
}

#[derive(Debug)]
struct MatchArm {
    pattern: Pat,
    guard: Option<(If, Box<Expr>)>,
    arrow: FatArrow,
    widget: Widget,
}

enum Attr {
    Doc(TokenStream2),
    Local(Ident),
    LocalRef(Ident),
    Root(Ident),
    Iterate(Ident),
    Watch(Ident),
    Track(Ident, Option<Box<Expr>>),
    BlockSignal(Ident, Vec<Ident>),
    Name(Ident, Ident),
    Transition(Ident, Ident),
    Wrap(Ident, Path),
    Chain(Ident, Box<Expr>),
    Template(Ident),
    TemplateChild(Ident),
}

struct Attrs {
    inner: Vec<Attr>,
}
