use crate::widgets::{TopLevelWidget, Widget};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

impl Parse for TopLevelWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.parse().ok();

        Ok(Self {
            inner: Widget::parse(input, attributes, None)?,
        })
    }
}
