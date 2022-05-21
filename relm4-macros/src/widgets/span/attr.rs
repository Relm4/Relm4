use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::Attr;

impl Spanned for Attr {
    fn span(&self) -> Span2 {
        match self {
            Self::Doc(tokens) => tokens.span(),
            Self::Local(ident) => ident.span(),
            Self::LocalRef(ident) => ident.span(),
            Self::Root(ident) => ident.span(),
            Self::Iterate(ident) => ident.span(),
            Self::Watch(ident) => ident.span(),
            Self::Track(ident, _) => ident.span(),
            Self::BlockSignal(ident, _) => ident.span(),
            Self::Name(ident, _) => ident.span(),
            Self::Transition(ident, _) => ident.span(),
        }
    }
}
