use crate::widgets::{
    AssignProperty, AssignPropertyAttr, ParseError, Property, PropertyName, PropertyType,
};

use super::{Format, FormatArgs, FormatAttributes, FormatLine, InlineFormat};

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
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let Property {
            blank_lines,
            name,
            ty,
        } = self;

        let empty_lines = (0..*blank_lines)
            .map(|_| FormatLine {
                indent_level: 0,
                line: "".to_owned(),
            })
            .collect();

        let mut prefix = name.inline_format();

        let (attrs, mut output) = match ty {
            PropertyType::Assign(assign) => {
                prefix += ": ";
                (
                    assign.format_attrs(indent_level),
                    assign.format(indent_level),
                )
            }
            PropertyType::SignalHandler(signal_handler) => {
                prefix += &signal_handler.format_args();
                prefix += " => ";
                (
                    signal_handler.format_attrs(indent_level),
                    signal_handler.format(indent_level),
                )
            }
            PropertyType::Widget(widget) => {
                if !matches!(name, PropertyName::RelmContainerExtAssign(_)) {
                    prefix += &widget.format_args();
                    prefix += " = ";
                }
                (
                    widget.format_attrs(indent_level),
                    widget.format(indent_level),
                )
            }
            PropertyType::ConditionalWidget(_) => todo!(),
            PropertyType::ParseError(error) => match error {
                ParseError::Ident((_ident, tokens)) => {
                    panic!("{tokens}");
                }
                ParseError::Path((_path, tokens)) => panic!("{tokens}"),
                ParseError::Generic(tokens) => panic!("{tokens}"),
            },
        };

        prefix.push_str(&output[0].line);
        output[0].line = prefix;

        output.last_mut().unwrap().line.push(',');

        [empty_lines, attrs, output].into_iter().flatten().collect()
    }
}

impl Format for AssignPropertyAttr {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        match &self {
            AssignPropertyAttr::None => Vec::new(),
            AssignPropertyAttr::Watch => vec![FormatLine {
                indent_level,
                line: "#[watch]".into(),
            }],
            AssignPropertyAttr::Track {
                expr,
                macro_generated,
            } => {
                if *macro_generated {
                    vec![FormatLine {
                        indent_level,
                        line: "#[track]".into(),
                    }]
                } else {
                    vec![FormatLine {
                        indent_level,
                        line: format!("#[track = \"{}\"]", expr.inline_format()),
                    }]
                }
            }
        }
    }
}

impl Format for AssignProperty {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        self.expr.format(indent_level)
    }
}

impl FormatAttributes for AssignProperty {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine> {
        self.attr.format(indent_level)
    }
}
