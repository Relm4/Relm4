use crate::widgets::{TopLevelWidget, ViewWidgets};

use super::{Format, FormatLine};

impl Format for ViewWidgets {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        self.top_level_widgets
            .iter()
            .flat_map(|widget| widget.format(ident_level))
            .collect()
    }
}

impl Format for TopLevelWidget {
    fn format(&self, ident_level: usize) -> Vec<FormatLine> {
        let mut output = Vec::new();

        if let Some(attr) = &self.root_attr {
            output.push(FormatLine {
                ident_level,
                line: format!("#[{attr}]"),
            })
        }

        output.extend(self.inner.format(ident_level));
        output
    }
}
