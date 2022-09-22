use proc_macro2::TokenStream as TokenStream2;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token::{And, Star};
use syn::{Error, Expr, Ident, Path, Token};

use crate::args::Args;
use crate::widgets::parse_util::{self, attr_twice_error};
use crate::widgets::{Attr, Attrs, ParseError, Properties, Widget, WidgetAttr, WidgetFunc};

type WidgetFuncInfo = (Option<And>, Option<Star>, WidgetFunc, Properties);

type AttributeInfo = (
    WidgetAttr,
    Option<TokenStream2>,
    Option<Ident>,
    Option<Path>,
);

impl Widget {
    pub(super) fn parse(
        input: ParseStream,
        attributes: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self, ParseError> {
        let (attr, doc_attr, new_name, assign_wrapper) = Self::process_attributes(attributes)?;
        // Check if first token is `mut`
        let mutable = input.parse().ok();

        // Look for name = Widget syntax
        let name_opt: Option<Ident> = if input.peek2(Token![=]) {
            if attr.is_local_attr() {
                return Err(input.error("When using the `local` or `local_ref` attributes you cannot rename the existing local variable.").into());
            }
            let name = input.parse()?;
            let _token: Token![=] = input.parse()?;
            Some(name)
        } else {
            None
        };

        let (ref_token, deref_token, func, properties) = Self::parse_widget_func(input)?;

        // Make sure that the name is only defined one.
        let mut name_set = name_opt.is_some();
        if new_name.is_some() {
            if name_set {
                return Err(Error::new(name_opt.unwrap().span(), "Widget name is specified more than once (attribute, assignment or local attribute).").into());
            }
            name_set = true;
        }

        if attr.is_local_attr() && name_set {
            return Err(Error::new(input.span(), "Widget name is specified more than once (attribute, assignment or local attribute).").into());
        }

        // Generate a name if no name was given.
        let (name, name_assigned_by_user) = if let Some(name) = name_opt.or(new_name) {
            (name, true)
        } else if attr.is_local_attr() {
            (Self::local_attr_name(&func)?, true)
        } else {
            (func.snake_case_name(), false)
        };

        let returned_widget = if input.peek(Token![->]) {
            let _arrow: Token![->] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Widget {
            doc_attr,
            attr,
            mutable,
            name,
            name_assigned_by_user,
            func,
            args,
            properties,
            assign_wrapper,
            ref_token,
            deref_token,
            returned_widget,
        })
    }

    pub(super) fn parse_for_container_ext(
        input: ParseStream,
        func: WidgetFunc,
        attributes: Option<Attrs>,
    ) -> Result<Self, ParseError> {
        let (attr, doc_attr, new_name, assign_wrapper) = Self::process_attributes(attributes)?;

        if let Some(wrapper) = assign_wrapper {
            return Err(Error::new(
                wrapper.span(),
                "Can't use wrapper types in container assignment.",
            )
            .into());
        }

        let properties = if input.peek(Token![,]) {
            Properties::default()
        } else {
            let inner = parse_util::braces(input)?;
            Properties::parse(&inner)
        };

        // Make sure that the name is only defined one.
        if attr.is_local_attr() {
            if let Some(name) = &new_name {
                return Err(Error::new(name.span(), "Widget name is specified more than once (attribute, assignment or local attribute).").into());
            }
        }
        // Generate a name
        let (name, name_assigned_by_user) = if let Some(name) = new_name {
            (name, true)
        } else if attr.is_local_attr() {
            (Self::local_attr_name(&func)?, true)
        } else {
            (func.snake_case_name(), false)
        };

        let ref_token = Some(And::default());

        Ok(Widget {
            doc_attr,
            attr,
            mutable: None,
            name,
            name_assigned_by_user,
            func,
            args: None,
            properties,
            assign_wrapper,
            ref_token,
            deref_token: None,
            returned_widget: None,
        })
    }

    fn process_attributes(attrs: Option<Attrs>) -> Result<AttributeInfo, ParseError> {
        if let Some(attrs) = attrs {
            let mut widget_attr = WidgetAttr::None;
            let mut doc_attr: Option<TokenStream2> = None;
            let mut name = None;
            let mut assign_wrapper = None;

            for attr in attrs.inner {
                let span = attr.span();
                match attr {
                    Attr::Local(_) => {
                        if widget_attr == WidgetAttr::None {
                            widget_attr = WidgetAttr::Local;
                        } else {
                            return Err(attr_twice_error(span).into());
                        }
                    }
                    Attr::LocalRef(_) => {
                        if widget_attr == WidgetAttr::None {
                            widget_attr = WidgetAttr::LocalRef;
                        } else {
                            return Err(attr_twice_error(span).into());
                        }
                    }
                    Attr::Doc(tokens) => {
                        if let Some(doc_tokens) = &mut doc_attr {
                            doc_tokens.extend(tokens);
                        } else {
                            doc_attr = Some(tokens);
                        }
                    }
                    Attr::Name(_, name_value) => {
                        if name.is_some() {
                            return Err(attr_twice_error(span).into());
                        }
                        name = Some(name_value);
                    }
                    Attr::Wrap(_, path) => {
                        if assign_wrapper.is_some() {
                            return Err(attr_twice_error(span).into());
                        }
                        assign_wrapper = Some(path.clone());
                    }
                    _ => {
                        return Err(Error::new(
                            attr.span(),
                            "Widgets can only have docs and `local`, `local_ref` or `root` as attribute.",
                        ).into());
                    }
                }
            }

            Ok((widget_attr, doc_attr, name, assign_wrapper))
        } else {
            Ok((WidgetAttr::None, None, None, None))
        }
    }

    // Make sure that the widget function is just a single identifier of the
    // local variable if a local attribute was set.
    fn local_attr_name(func: &WidgetFunc) -> Result<Ident, ParseError> {
        if let Some(name) = func.path.get_ident() {
            Ok(name.clone())
        } else {
            Err(Error::new(
                func.path.span(),
                "Expected identifier due to the `local` or `local_ref` attribute.",
            )
            .into())
        }
    }

    /// Parse information related to the widget function.
    fn parse_widget_func(input: ParseStream) -> Result<WidgetFuncInfo, ParseError> {
        // Look for &
        let ref_token = input.parse().ok();

        // Look for *
        let deref_token = input.parse().ok();

        let func = WidgetFunc::parse(input)?;

        let properties = if input.peek(Token![,]) {
            Properties::default()
        } else {
            let inner = parse_util::braces(input)?;
            Properties::parse(&inner)
        };

        Ok((ref_token, deref_token, func, properties))
    }
}
