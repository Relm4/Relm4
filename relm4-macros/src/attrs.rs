use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Error, Ident, Path, Result, Token, Visibility,
};

use crate::util::default_relm4_path;

enum AttributeType {
    None,
    Named,
    Unnamed { span: Span },
}

pub struct Attrs {
    /// Keeps information about visibility of the widget
    pub visibility: Option<Visibility>,

    /// Path to relm4
    ///
    /// Defaults to `::relm4`
    pub relm4_path: Path,
}

impl Attrs {
    fn new() -> Self {
        Attrs {
            visibility: None,
            relm4_path: default_relm4_path(),
        }
    }
}

impl Parse for Attrs {
    /// Rules for parsing attributes.
    ///
    /// 1. It's fine if visibility is used unnamed so `#[widget(pub)]` must be valid but thats the only case.
    /// 2. Widget visibility might be named `#[widget(visibility = pub)]`.
    /// 3. `relm4` argument must be named. Always.
    ///
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Attrs::new();
        let mut attrs_type = AttributeType::None;

        // Allows to track if relm4 path was already set.
        // You can't set relm4 path twice.
        //
        // ```rust, ignore
        // #[widget(relm4 = ::my::path, relm4 = ::my::other::path ) ]
        // ```
        // is illegal.
        let mut relm4_path_set = false;

        let mixed_use_error_message =
            "You can't mix named and unnamed arguments while using `relm4_macros::widget`. \n\
            \n\
            You can use one of\n\
            \n\
            1. `#[relm4_macros::widget()]` to define widget with private visibility\n\
            2. `#[relm4_macros::widget(pub)]` to define widget with public visibility\n\
            3. `#[relm4_macros::widget(visibility = pub)]` to define widget with public visibility and potentially other arguments\n\
            \n\
            Please use `visibility = pub` to fix this error";

        while !input.is_empty() {
            if input.peek(Token![pub]) {
                if matches!(attrs_type, AttributeType::Named) {
                    return Err(input.error(mixed_use_error_message));
                }
                if attrs.visibility.is_some() {
                    return Err(input.error("You can't assign visibility twice"));
                }
                let pub_vis: Visibility = input.parse()?;

                attrs_type = AttributeType::Unnamed {
                    span: pub_vis.span(),
                };
                attrs.visibility = Some(pub_vis);
            } else {
                let ident: Ident = input.parse()?;
                let _eq: Token![=] = input.parse()?;

                if ident == "visibility" {
                    let pub_vis: Visibility = input.parse()?;

                    if let AttributeType::Unnamed { span } = attrs_type {
                        return Err(Error::new(span, mixed_use_error_message));
                    }
                    if attrs.visibility.is_some() {
                        return Err(Error::new(
                            pub_vis.span(),
                            "You can't assign visibility twice",
                        ));
                    }

                    attrs.visibility = Some(pub_vis);
                    attrs_type = AttributeType::Named;
                } else if ident == "relm4" {
                    let path: Path = input.parse()?;

                    if let AttributeType::Unnamed { span } = attrs_type {
                        return Err(Error::new(span, mixed_use_error_message));
                    }
                    if relm4_path_set {
                        return Err(Error::new(path.span(), "You can't assign relm4 path twice"));
                    }

                    attrs.relm4_path = path;
                    relm4_path_set = true;
                    attrs_type = AttributeType::Named;
                } else {
                    return Err(input
                        .error("Unknown argument. Valid arguments are: `visibility` or `relm4`"));
                }
            }

            if input.peek(Token![,]) {
                let comma: Token![,] = input.parse()?;
                if input.is_empty() {
                    // We've just consumed last token in stream (which is comma) and that's wrong
                    return Err(Error::new(comma.span, "Unexpected comma found"));
                }
            }
        }

        Ok(attrs)
    }
}
