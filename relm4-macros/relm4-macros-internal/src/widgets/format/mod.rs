mod args;
mod conditional_widget;
mod property;
mod returned_widget;
mod signal_handler;
mod syn;
mod view_widgets;
mod widget;

#[derive(Debug)]
pub struct FormatLine {
    pub indent_level: usize,
    pub line: String,
}

pub trait Format {
    fn format(&self, indent_level: usize) -> Vec<FormatLine>;
}

pub trait FormatAttributes {
    fn format_attrs(&self, indent_level: usize) -> Vec<FormatLine>;
}

pub trait FormatArgs {
    fn format_args(&self) -> String;
}

pub trait InlineFormat {
    fn inline_format(&self) -> String;
}

impl<T: Format> InlineFormat for T {
    fn inline_format(&self) -> String {
        let output = self.format(0);
        output.into_iter().map(|s| s.line).collect()
    }
}
