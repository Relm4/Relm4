use crate::widgets::{AssignProperty, AssignPropertyAttr, Property, PropertyName, PropertyType};

use super::{Format, FormatAttributes, FormatLine, InlineFormat};

impl InlineFormat for PropertyName {
    fn inline_format(&self) -> String {
        match self {
            PropertyName::Ident(ident) => ident.to_string(),
            PropertyName::Path(path) => path.inline_format(),
            PropertyName::RelmContainerExtAssign(_) => String::new(),
        }
    }
}

impl Format for Property {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        let mut prefix = self.name.inline_format();

        let (attrs, mut output) = match &self.ty {
            PropertyType::Assign(assign) => {
                prefix += ": ";
                (assign.format_attrs(ident_level), assign.format(ident_level))
            }
            PropertyType::SignalHandler(signal_handler) => {
                prefix += " => ";
                (
                    signal_handler.format_attrs(ident_level),
                    signal_handler.format(ident_level),
                )
            }
            PropertyType::Widget(widget) => {
                if !matches!(&self.name, PropertyName::RelmContainerExtAssign(_)) {
                    prefix += "= ";
                }
                (widget.format_attrs(ident_level), widget.format(ident_level))
            }
            PropertyType::ConditionalWidget(_) => todo!(),
            PropertyType::ParseError(_) => todo!(),
        };

        prefix.push_str(&output[0].line);
        output[0].line = prefix;

        output.last_mut().unwrap().line.push(',');

        [attrs, output].into_iter().flatten().collect()
    }
}

impl Format for AssignPropertyAttr {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        match &self {
            AssignPropertyAttr::None => Vec::new(),
            AssignPropertyAttr::Watch => vec![FormatLine {
                ident_level,
                line: "#[watch]".into(),
            }],
            AssignPropertyAttr::Track {
                expr,
                macro_generated,
            } => {
                if *macro_generated {
                    vec![FormatLine {
                        ident_level,
                        line: "#[track]".into(),
                    }]
                } else {
                    vec![FormatLine {
                        ident_level,
                        line: format!("#[track = \"{}\"]", expr.inline_format()),
                    }]
                }
            }
        }
    }
}

impl Format for AssignProperty {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        self.expr.format(ident_level)
    }
}

impl FormatAttributes for AssignProperty {
    fn format_attrs(&self, ident_level: usize) -> Vec<FormatLine> {
        self.attr.format(ident_level)
    }
}
