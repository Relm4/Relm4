use crate::widgets::{AssignProperty, Property, PropertyName, PropertyType};

use super::{Format, FormatLine, InlineFormat};

impl InlineFormat for PropertyName {
    fn inline_format(&self) -> String {
        match self {
            PropertyName::Ident(ident) => format!("{ident}: "),
            PropertyName::Path(path) => format!("{}: ", path.inline_format()),
            PropertyName::RelmContainerExtAssign(_) => String::new(),
        }
    }
}

impl Format for Property {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        let mut prefix = self.name.inline_format();

        let mut output = match &self.ty {
            PropertyType::Assign(assign) => assign.format(ident_level),
            PropertyType::SignalHandler(signal_handler) => signal_handler.format(ident_level),
            PropertyType::Widget(widget) => widget.format(ident_level),
            PropertyType::ConditionalWidget(_) => todo!(),
            PropertyType::ParseError(_) => todo!(),
        };

        prefix.push_str(&output[0].line);
        output[0].line = prefix;

        output.last_mut().unwrap().line.push(',');

        output
    }
}

impl Format for AssignProperty {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        self.expr.format(ident_level)
    }
}
