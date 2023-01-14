use crate::widgets::{Widget, WidgetAttr, WidgetFunc, WidgetTemplateAttr};

use super::{Format, FormatLine, InlineFormat};

impl Format for WidgetAttr {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        match self {
            WidgetAttr::None => Vec::new(),
            WidgetAttr::Local => vec![FormatLine {
                ident_level,
                line: "#[local]".to_string(),
            }],
            WidgetAttr::LocalRef => vec![FormatLine {
                ident_level,
                line: "#[local_ref]".to_string(),
            }],
        }
    }
}

impl Format for WidgetTemplateAttr {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        match self {
            WidgetTemplateAttr::None => Vec::new(),
            WidgetTemplateAttr::Template => vec![FormatLine {
                ident_level,
                line: "#[template]".to_string(),
            }],
            WidgetTemplateAttr::TemplateChild => vec![FormatLine {
                ident_level,
                line: "#[template_child]".to_string(),
            }],
        }
    }
}

impl InlineFormat for WidgetFunc {
    fn inline_format(&self) -> String {
        let mut line = String::new();

        line.push_str(&self.path.inline_format());
        line.push_str(" {");

        line
    }
}

impl Format for Widget {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        let mut output = Vec::new();

        output.extend(self.doc_attr.as_ref().map(|tokens| FormatLine {
            ident_level,
            line: tokens.to_string(),
        }));

        if self.name_assigned_by_user {
            output.push(FormatLine {
                ident_level,
                line: format!("#[name = \"{}\"]", self.name),
            });
        }

        output.extend(self.attr.format(ident_level));
        output.extend(self.template_attr.format(ident_level));

        let mut line = String::new();
        line.push_str(&self.func.inline_format());

        output.push(FormatLine { ident_level, line });

        for props in &self.properties.properties {
            output.extend(props.format(ident_level + 1));
        }

        output.push(FormatLine {
            ident_level,
            line: "}".to_string(),
        });

        output
    }
}
