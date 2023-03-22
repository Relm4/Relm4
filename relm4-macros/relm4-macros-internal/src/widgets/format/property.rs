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
            comments,
            name,
            ty,
        } = self;

        let comments = comments
            .iter()
            .map(|c| FormatLine {
                indent_level,
                line: c.to_string(),
            })
            .collect();

        let empty_lines = (0..*blank_lines)
            .map(|_| FormatLine {
                indent_level: 0,
                line: "".to_owned(),
            })
            .collect();

        let mut prefix = name.inline_format();

        let (attrs, mut output) = match ty {
            PropertyType::Assign(assign) => {
                prefix += &assign.format_args();
                prefix += ": ";
                (
                    assign.format_attrs(indent_level),
                    assign.format(indent_level),
                )
            }
            PropertyType::SignalHandler(signal_handler) => {
                prefix += &signal_handler.format_args();
                prefix += " => ";
                (Vec::new(), signal_handler.format(indent_level))
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
            PropertyType::ConditionalWidget(conditional_widget) => {
                if !matches!(name, PropertyName::RelmContainerExtAssign(_)) {
                    prefix += &conditional_widget.format_args();
                    prefix += " = ";
                }
                (
                    conditional_widget.format_attrs(indent_level),
                    conditional_widget.format(indent_level),
                )
            }
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

        [empty_lines, attrs, comments, output]
            .into_iter()
            .flatten()
            .collect()
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
                        line: format!("#[track({})]", expr.inline_format()),
                    }]
                }
            }
        }
    }
}

impl FormatArgs for AssignProperty {
    fn format_args(&self) -> String {
        let mut output = self
            .args
            .as_ref()
            .map(|args| format!("[{}]", args.inline_format()))
            .unwrap_or_default();

        if self.optional_assign {
            output += "?";
        }

        output
    }
}

impl Format for AssignProperty {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        self.expr.format(indent_level)
    }
}

impl FormatAttributes for AssignProperty {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine> {
        let AssignProperty {
            attr,
            iterative,
            block_signals,
            chain,
            ..
        } = self;
        let mut output = attr.format(indent_level);

        if !block_signals.is_empty() {
            let line: String = block_signals.iter().map(|i| format!("{i}, ")).collect();
            let line = format!("#[block_signal({})]", line.trim_end_matches(", "));
            output.push(FormatLine { indent_level, line });
        }

        if *iterative {
            let line = "#[iterate]".into();
            output.push(FormatLine { indent_level, line });
        }

        if let Some(chain) = chain {
            let line = format!("#[chain({})]", chain.inline_format());
            output.push(FormatLine { indent_level, line });
        }

        output
    }
}
