use syn::parse::ParseStream;

use crate::util;
use crate::widgets::{
    parse_util, Attr, Attrs, Properties, Property, PropertyName, PropertyType, TopLevelWidget,
    Widget, WidgetAttr, WidgetFunc, WidgetTemplateAttr,
};

impl TopLevelWidget {
    pub(super) fn parse(input: ParseStream<'_>) -> Self {
        let attributes: Option<Attrs> = input.parse().ok();

        // Look for #[root] attribute and remove it from the list if it exists
        let (attributes, root_attr) = if let Some(prev_attributes) = attributes {
            let mut attributes = Attrs {
                inner: Vec::with_capacity(prev_attributes.inner.len()),
            };
            let mut root_attr = None;
            for attr in prev_attributes.inner {
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

        let inner = match Widget::parse(input, attributes, None) {
            Ok(inner) => inner,
            Err(err) => Widget {
                doc_attr: None,
                attr: WidgetAttr::None,
                template_attr: WidgetTemplateAttr::None,
                mutable: None,
                name: parse_util::string_to_snake_case("incorrect_top_level_widget"),
                name_assigned_by_user: false,
                func: WidgetFunc {
                    path: util::strings_to_path(&["gtk", "Box"]),
                    args: None,
                    method_chain: None,
                    ty: None,
                },
                args: None,
                properties: Properties {
                    properties: vec![Property {
                        name: PropertyName::Ident(parse_util::string_to_snake_case(
                            "invalid_property",
                        )),
                        ty: PropertyType::ParseError(err),
                    }],
                },
                assign_wrapper: None,
                ref_token: None,
                deref_token: None,
                returned_widget: None,
            },
        };

        Self { root_attr, inner }
    }
}
