use crate::widgets::{SignalHandler, SignalHandlerVariant};

use super::{Format, FormatArgs, FormatAttributes, FormatLine, InlineFormat};

impl FormatAttributes for SignalHandler {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine> {
        if let Some(id) = &self.handler_id {
            vec![FormatLine {
                indent_level,
                line: format!("#[handler_id = \"{id}\"]"),
            }]
        } else {
            Vec::new()
        }
    }
}

impl Format for SignalHandler {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        match &self.inner {
            SignalHandlerVariant::Expr(expr) => expr.format(indent_level),
            SignalHandlerVariant::Closure(closure) => closure.closure.format(indent_level),
        }
    }
}

impl FormatArgs for SignalHandler {
    fn format_args(&self) -> String {
        match &self.inner {
            SignalHandlerVariant::Expr(_) => "".into(),
            SignalHandlerVariant::Closure(closure) => closure
                .args
                .as_ref()
                .map(|args| format!("[{}]", args.inline_format()))
                .unwrap_or_default(),
        }
    }
}
