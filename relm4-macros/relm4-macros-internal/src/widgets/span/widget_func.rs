use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::WidgetFunc;

impl Spanned for WidgetFunc {
    fn span(&self) -> Span2 {
        self.path.span()
    }
}
