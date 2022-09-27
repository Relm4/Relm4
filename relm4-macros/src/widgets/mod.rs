use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::punctuated::Punctuated;
use syn::token::{Else, FatArrow, If, Match, Mut};
use syn::{token, Expr, ExprClosure, Ident, MethodTurbofish, Pat, Path};

use crate::args::Args;

mod gen;
mod parse;
mod parse_util;
mod span;

pub(super) struct ViewWidgets {
    pub(super) span: Span2,
    pub(super) top_level_widgets: Vec<TopLevelWidget>,
}

pub(super) struct TopLevelWidget {
    pub(super) root_attr: Option<Ident>,
    pub(super) inner: Widget,
}

enum PropertyType {
    Assign(AssignProperty),
    SignalHandler(SignalHandler),
    Widget(Widget),
    ConditionalWidget(ConditionalWidget),
    ParseError(ParseError),
}

enum ParseError {
    Ident((Ident, TokenStream2)),
    Path((Path, TokenStream2)),
    Generic(TokenStream2),
}

enum AssignPropertyAttr {
    None,
    Watch,
    /// The bool indicated whether the model type needs to
    /// be pasted in front of the track expression.
    Track((TokenStream2, bool)),
}

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

struct SignalHandler {
    inner: SignalHandlerVariant,
    handler_id: Option<Ident>,
}

enum SignalHandlerVariant {
    Expr(Expr),
    Closure(ClosureSignalHandler),
}

struct ClosureSignalHandler {
    closure: ExprClosure,
    args: Option<Args<Expr>>,
}

enum PropertyName {
    Ident(Ident),
    Path(Path),
    RelmContainerExtAssign,
}

struct Property {
    /// Either a path or just an ident
    name: PropertyName,
    ty: PropertyType,
}

#[derive(Default)]
struct Properties {
    properties: Vec<Property>,
}

/// The function that initializes the widget.
///
/// This might be a real function or just something like `gtk::Label`.
struct WidgetFunc {
    path: Path,
    args: Option<Punctuated<Expr, token::Comma>>,
    method_chain: Option<Punctuated<WidgetFuncMethod, token::Dot>>,
    ty: Option<Path>,
}

struct WidgetFuncMethod {
    ident: Ident,
    turbofish: Option<MethodTurbofish>,
    args: Option<Punctuated<Expr, token::Comma>>,
}

pub(super) struct Widget {
    doc_attr: Option<TokenStream2>,
    attr: WidgetAttr,
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

#[derive(PartialEq)]
enum WidgetAttr {
    None,
    Local,
    LocalRef,
}

struct ReturnedWidget {
    name: Ident,
    ty: Option<Path>,
    properties: Properties,
    is_optional: bool,
}

struct ConditionalWidget {
    doc_attr: Option<TokenStream2>,
    transition: Option<Ident>,
    assign_wrapper: Option<Path>,
    name: Ident,
    args: Option<Args<Expr>>,
    branches: ConditionalBranches,
}

enum ConditionalBranches {
    If(Vec<IfBranch>),
    Match((Match, Box<Expr>, Vec<MatchArm>)),
}

enum IfCondition {
    If(If, Expr),
    ElseIf(Else, If, Expr),
    Else(Else),
}

struct IfBranch {
    cond: IfCondition,
    widget: Widget,
}

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
}

struct Attrs {
    inner: Vec<Attr>,
}
