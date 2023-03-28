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
            | Self::Watch(ident, _)
            | Self::Track(ident, _, _)
            | Self::BlockSignal(ident, _)
            | Self::Name(ident, _)
            | Self::Transition(ident, _)
            | Self::Chain(ident, _)
            | Self::Template(ident)
            | Self::TemplateChild(ident)
            | Self::Wrap(ident, _) => ident.span(),
        }
    }
}
