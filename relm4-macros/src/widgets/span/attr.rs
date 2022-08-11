use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::Attr;

impl Spanned for Attr {
    fn span(&self) -> Span2 {
        match self {
            Self::Doc(tokens) => tokens.span(),
            Self::Local(ident)
            | Self::LocalRef(ident)
            | Self::Root(ident)
            | Self::Iterate(ident)
            | Self::Watch(ident)
            | Self::Track(ident, _)
            | Self::BlockSignal(ident, _)
            | Self::Name(ident, _)
            | Self::Transition(ident, _)
            | Self::Wrap(ident, _) => ident.span(),
        }
    }
}
