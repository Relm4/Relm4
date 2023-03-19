use quote::ToTokens;

use crate::widgets::{RefToken, Widget, WidgetAttr, WidgetFunc, WidgetTemplateAttr};

use super::{syn::call_rustfmt, Format, FormatArgs, FormatAttributes, FormatLine, InlineFormat};

impl Format for WidgetAttr {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        match self {
            WidgetAttr::None => Vec::new(),
            WidgetAttr::Local => vec![FormatLine {
                indent_level,
                line: "#[local]".to_string(),
            }],
            WidgetAttr::LocalRef => vec![FormatLine {
                indent_level,
                line: "#[local_ref]".to_string(),
            }],
        }
    }
}

impl Format for WidgetTemplateAttr {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        match self {
            WidgetTemplateAttr::None => Vec::new(),
            WidgetTemplateAttr::Template => vec![FormatLine {
                indent_level,
                line: "#[template]".to_string(),
            }],
            WidgetTemplateAttr::TemplateChild => vec![FormatLine {
                indent_level,
                line: "#[template_child]".to_string(),
            }],
        }
    }
}

impl InlineFormat for WidgetFunc {
    fn inline_format(&self) -> String {
        let WidgetFunc {
            path,
            args,
            method_chain,
            ty,
        } = self;

        let mut line = String::new();

        line.push_str(&path.inline_format());
        if let Some(args) = args {
            line.push_str(&args.into_token_stream().to_string());
        }
        if let Some(methods) = method_chain {
            line.push_str(&methods.into_token_stream().to_string());
        }

        let mut line = call_rustfmt(line, "const T: X = ", ";");
        if let Some(ty) = ty {
            line.push_str(" -> ");
            line.push_str(&ty.to_token_stream().to_string());
        }
        line.push_str(" {");

        line
    }
}

impl FormatAttributes for Widget {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine> {
        let Widget {
            doc_attr,
            attr,
            template_attr,
            name,
            name_assigned_by_user,
            assign_wrapper,
            ..
        } = &self;
        let mut output = Vec::new();

        output.extend(doc_attr.as_ref().map(|tokens| FormatLine {
            indent_level,
            line: tokens.to_string(),
        }));

        if *name_assigned_by_user {
            output.push(FormatLine {
                indent_level,
                line: format!("#[name = \"{}\"]", name),
            });
        }

        if let Some(wrapper) = assign_wrapper {
            output.push(FormatLine {
                indent_level,
                line: format!("#[wrap({})]", wrapper.inline_format()),
            });
        }

        output.extend(attr.format(indent_level));
        output.extend(template_attr.format(indent_level));

        output
    }
}

impl FormatArgs for Widget {
    fn format_args(&self) -> String {
        self.args
            .as_ref()
            .map(|args| format!("[{}]", args.inline_format()))
            .unwrap_or_default()
    }
}

impl Format for Widget {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let Widget {
            mutable,
            func,
            properties,
            ref_token,
            deref_token,
            returned_widget,
            ..
        } = self;
        let mut output = Vec::new();

        let mut line = String::new();
        if mutable.is_some() {
            line.push_str("mut ");
        }

        if deref_token.is_some() {
            line.push('*');
        }
        if let RefToken::Some(_) = ref_token {
            line.push('&');
        }

        line.push_str(&func.inline_format());

        output.push(FormatLine { indent_level, line });

        for props in &properties.properties {
            output.extend(props.format(indent_level + 1));
        }

        if let Some(returned_widget) = returned_widget {
            output.push(FormatLine {
                indent_level,
                line: "} -> {".to_string(),
            });
            output.extend(returned_widget.format(indent_level + 1));
            output.push(FormatLine {
                indent_level,
                line: "}".to_string(),
            });
        } else {
            output.push(FormatLine {
                indent_level,
                line: "}".to_string(),
            });
        }

        output
    }
}
