use proc_macro2::Span as Span2;
use syn::spanned::Spanned;

use crate::widgets::WidgetFunc;

impl WidgetFunc {
    pub(crate) fn span(&self) -> Span2 {
        self.path.span()
    }
}
