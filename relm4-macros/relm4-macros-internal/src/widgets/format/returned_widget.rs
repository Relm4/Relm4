use crate::widgets::ReturnedWidget;

use super::Format;

impl Format for ReturnedWidget {
    fn format(&self, _indent_level: usize) -> Vec<super::FormatLine> {
        todo!("Returned widgets are not supported yet");
    }
}
