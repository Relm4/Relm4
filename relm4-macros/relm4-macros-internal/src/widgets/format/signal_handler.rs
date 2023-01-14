use crate::widgets::{SignalHandler, SignalHandlerVariant};

use super::{Format, FormatAttributes, FormatLine};

impl FormatAttributes for SignalHandler {
    fn format_attrs(&self, ident_level: usize) -> Vec<FormatLine> {
        if let Some(id) = &self.handler_id {
            vec![FormatLine {
                ident_level,
                line: format!("#[handler_id = \"{id}\"]"),
            }]
        } else {
            Vec::new()
        }
    }
}

impl Format for SignalHandler {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        match &self.inner {
            SignalHandlerVariant::Expr(expr) => expr.format(ident_level),
            SignalHandlerVariant::Closure(closure) => closure.closure.format(ident_level),
        }
    }
}
