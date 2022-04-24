use crate::widgets::{Attr, Attrs, TopLevelWidget, Widget};
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

impl Parse for TopLevelWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes: Option<Attrs> = input.parse().ok();

        // Look for #[root] attribute and remove it from the list if it exists
        let (attributes, root_attr) = if let Some(prev_attributes) = attributes {
            let mut attributes = Attrs {
                inner: Vec::with_capacity(prev_attributes.inner.len()),
            };
            let mut root_attr = None;
            for attr in prev_attributes.inner.into_iter() {
                match attr {
                    Attr::Root(ident) => {
                        // Save root attribute and don't push it to the new list
                        root_attr = Some(ident);
                    }
                    _ => attributes.inner.push(attr),
                }
            }
            (Some(attributes), root_attr)
        } else {
            (None, None)
        };

        Ok(Self {
            root_attr,
            inner: Widget::parse(input, attributes, None)?,
        })
    }
}
