use crate::widgets::{SignalHandler, SignalHandlerVariant};

use super::{Format, FormatLine};

impl Format for SignalHandler {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        match &self.inner {
            SignalHandlerVariant::Expr(expr) => expr.format(ident_level),
            SignalHandlerVariant::Closure(closure) => closure.closure.format(ident_level),
        }
    }
}
