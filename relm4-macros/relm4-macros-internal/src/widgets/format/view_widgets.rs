use crate::widgets::{TopLevelWidget, ViewWidgets};

use super::{Format, FormatAttributes, FormatLine};

impl Format for ViewWidgets {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        self.top_level_widgets
            .iter()
            .flat_map(|widget| widget.format(indent_level))
            .collect()
    }
}

impl Format for TopLevelWidget {
    fn format(&self, indent_level: usize) -> Vec<FormatLine> {
        let mut output = Vec::new();

        if let Some(attr) = &self.root_attr {
            output.push(FormatLine {
                indent_level,
                line: format!("#[{attr}]"),
            })
        }

        output.extend(self.inner.format_attrs(indent_level));
        output.extend(self.inner.format(indent_level));

        let last = output.last_mut().unwrap();
        last.line += ",";

        output
    }
}
