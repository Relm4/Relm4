use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::PropertyName;

impl Spanned for PropertyName {
    fn span(&self) -> Span2 {
        match self {
            PropertyName::Ident(ident) => ident.span(),
            PropertyName::Path(path) => path.span(),
            PropertyName::RelmContainerExtAssign => Span2::call_site(),
        }
    }
}
