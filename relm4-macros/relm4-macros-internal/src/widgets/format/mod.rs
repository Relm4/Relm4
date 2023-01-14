mod property;
mod signal_handler;
mod syn;
mod view_widgets;
mod widget;

pub struct FormatLine {
    pub ident_level: usize,
    pub line: String,
}

pub trait Format {
    fn format(&self, ident_level: usize) -> Vec<FormatLine>;
}

pub trait InlineFormat {
    fn inline_format(&self) -> String;
}
