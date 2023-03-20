use quote::ToTokens;

use crate::widgets::{RefToken, Widget, WidgetAttr, WidgetFunc, WidgetTemplateAttr};

use super::{
    syn::call_rustfmt_to_lines, Format, FormatArgs, FormatAttributes, FormatLine, InlineFormat,
};

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

impl Format for WidgetFunc {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let WidgetFunc {
            path,
            args,
            method_chain,
            ty,
        } = self;

        let mut input = String::new();

        input.push_str(&path.inline_format());
        if let Some(args) = args {
            input.push('(');
            input.push_str(&args.into_token_stream().to_string());
            input.push(')');
        }

        if let Some(methods) = method_chain {
            input.push('.');
            input.push_str(&methods.into_token_stream().to_string());
        }

        let mut lines = call_rustfmt_to_lines(input, "const T: X = ", ";", indent_level);
        let last = &mut lines.last_mut().unwrap();
        if let Some(ty) = ty {
            last.line.push_str(" -> ");
            last.line.push_str(&ty.inline_format());
        }

        lines
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
            line: format!("///{}", tokens.to_string().trim_matches('"')),
        }));

        if *name_assigned_by_user && !attr.is_local_attr() {
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

        let is_empty = properties.properties.is_empty();

        let mut output = {
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

            let mut output = func.format(indent_level);
            let first = output.first_mut().unwrap();
            first.line = line + &first.line;

            if !is_empty {
                first.line += " {";
            }

            output
        };

        for props in &properties.properties {
            output.extend(props.format(indent_level + 1));
        }

        if let Some(returned_widget) = returned_widget {
            output.extend(returned_widget.format(indent_level + 1));
        }

        if !is_empty {
            output.push(FormatLine {
                indent_level,
                line: "}".to_owned(),
            });
        }

        output
    }
}
