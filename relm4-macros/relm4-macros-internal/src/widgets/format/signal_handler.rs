use crate::widgets::{SignalHandler, SignalHandlerVariant};

use super::{Format, FormatArgs, FormatLine, InlineFormat};

impl Format for SignalHandler {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let mut output = match &self.inner {
            SignalHandlerVariant::Expr(expr) => expr.format(indent_level),
            SignalHandlerVariant::Closure(closure) => closure.closure.format(indent_level),
        };

        if let Some(id) = &self.handler_id {
            let last = output.last_mut().unwrap();
            last.line += &format!(" @{id}");
        }

        output
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
