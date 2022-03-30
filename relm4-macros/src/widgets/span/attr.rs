use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::Attr;

impl Spanned for Attr {
    fn span(&self) -> Span2 {
        match self {
            Self::Local(ident) => ident.span(),
            Self::Iterate(ident) => ident.span(),
            Self::Watch(ident) => ident.span(),
            Self::Track(ident, _) => ident.span(),
        }
    }
}
