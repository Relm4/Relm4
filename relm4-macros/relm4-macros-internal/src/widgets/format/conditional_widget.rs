use crate::widgets::{ConditionalBranches, ConditionalWidget, IfBranch, IfCondition, MatchArm};

use super::{Format, FormatArgs, FormatAttributes, FormatLine, InlineFormat};

impl FormatArgs for ConditionalWidget {
    fn format_args(&self) -> String {
        self.args
            .as_ref()
            .map(|args| format!("[{}]", args.inline_format()))
            .unwrap_or_default()
    }
}

impl FormatAttributes for ConditionalWidget {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine> {
        let ConditionalWidget {
            doc_attr,
            transition,
            assign_wrapper,
            name_assigned_by_user,
            name,
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

        if let Some(transition) = transition {
            output.push(FormatLine {
                indent_level,
                line: format!("#[transition({})]", transition.to_string()),
            });
        }

        output
    }
}

impl Format for ConditionalWidget {
    fn format(&self, indent_level: usize) -> Vec<super::FormatLine> {
        let ConditionalWidget { branches, .. } = self;

        match branches {
            ConditionalBranches::If(if_branches) => if_branches.format(indent_level),
            ConditionalBranches::Match((_, expr, march_arms)) => {
                let mut output = expr.format(indent_level);
                output[0].line = "match ".to_owned() + &output[0].line + " {";

                output.extend(march_arms.format(indent_level + 1));

                output.push(FormatLine {
                    indent_level,
                    line: "}".into(),
                });
                output
            }
        }
    }
}

impl Format for Vec<IfBranch> {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let mut output = Vec::new();

        for branch in self {
            let IfBranch { cond, widget } = branch;
            let lines = match cond {
                IfCondition::If(_, condition) => {
                    let mut lines = condition.format(indent_level);
                    lines[0].line = "if ".to_owned() + &lines[0].line + " {";
                    lines
                }
                IfCondition::ElseIf(_, _, condition) => {
                    let mut lines = condition.format(indent_level);
                    lines[0].line = "} else if ".to_owned() + &lines[0].line + " {";
                    lines
                }
                IfCondition::Else(_) => {
                    vec![FormatLine {
                        indent_level,
                        line: "} else {".into(),
                    }]
                }
            };
            output.extend(lines);

            output.extend(widget.format(indent_level + 1));
        }

        output.push(FormatLine {
            indent_level,
            line: "}".into(),
        });
        output
    }
}

impl Format for Vec<MatchArm> {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let mut output = Vec::new();

        for arm in self {
            let MatchArm {
                widget,
                pattern,
                guard,
                ..
            } = arm;

            let mut input = (pattern, guard).format(indent_level);
            let last = input.last_mut().unwrap();
            last.line += " => {";

            output.extend(input);

            output.extend(widget.format(indent_level + 1));
            output.push(FormatLine {
                indent_level,
                line: "},".into(),
            });
        }

        let last = output.last_mut().unwrap();
        last.line = last.line.trim_end_matches(',').to_owned();

        output
    }
}
