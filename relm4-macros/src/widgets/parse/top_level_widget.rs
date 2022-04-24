use crate::widgets::{Attr, Attrs, TopLevelWidget, Widget};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

impl Parse for TopLevelWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attributes: Option<Attrs> = input.parse().ok();

        // Look for #[root] attribute and remove it from the list if it exists
        let is_root = if let Some(attributes) = &mut attributes {
            let root_pos = attributes
                .inner
                .iter()
                .position(|attr| matches!(attr, Attr::Root(_)));
            if let Some(root_pos) = root_pos {
                attributes.inner.swap_remove(root_pos);
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(Self {
            is_root,
            inner: Widget::parse(input, attributes, None)?,
        })
    }
}
