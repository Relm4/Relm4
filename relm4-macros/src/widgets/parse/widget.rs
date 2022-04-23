use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced, parenthesized,
    parse::{ParseBuffer, ParseStream},
    spanned::Spanned,
    token,
    token::{And, Star},
    Error, Expr, Ident, Result, Token,
};

use crate::widgets::{
    util::attr_twice_error, Attr, Attrs, Properties, Widget, WidgetFunc, WidgetFuncPath,
};
use crate::{args::Args, widgets::WidgetAttr};

type WidgetFuncInfo = (
    // For `Some(widget)`
    Option<Ident>,
    Option<And>,
    Option<Star>,
    WidgetFunc,
    Properties,
);

impl Widget {
    pub(super) fn parse(
        input: ParseStream,
        attributes: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self> {
        let (attr, doc_attr) = Self::process_attributes(attributes)?;
        // Check if first token is `mut`
        let mutable = input.parse().ok();

        // Look for name = Widget syntax
        let name_opt: Option<Ident> = if input.peek2(Token![=]) {
            if attr.is_local_attr() {
                return Err(input.error("When using the `local` or `local_ref` attributes you cannot rename the existing local variable."));
            } else {
                let name = input.parse()?;
                let _token: Token![=] = input.parse()?;
                Some(name)
            }
        } else {
            None
        };

        let (wrapper, ref_token, deref_token, func, properties) = Self::parse_widget_func(input)?;

        // Generate a name if no name was given.
        let name = if let Some(name) = name_opt {
            name
        } else if attr.is_local_attr() {
            Self::local_attr_name(&func)?
        } else {
            func.snake_case_name()
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
            func,
            args,
            properties,
            wrapper,
            ref_token,
            deref_token,
            returned_widget,
        })
    }

    pub(super) fn parse_for_container_ext(
        input: ParseStream,
        func: WidgetFunc,
        attributes: Option<Attrs>,
    ) -> Result<Self> {
        let (attr, doc_attr) = Self::process_attributes(attributes)?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        // Generate a name
        let name = if attr.is_local_attr() {
            Self::local_attr_name(&func)?
        } else {
            func.snake_case_name()
        };

        let ref_token = Some(And::default());

        Ok(Widget {
            doc_attr,
            attr,
            mutable: None,
            name,
            func,
            args: None,
            properties,
            wrapper: None,
            ref_token,
            deref_token: None,
            returned_widget: None,
        })
    }

    fn process_attributes(attrs: Option<Attrs>) -> Result<(WidgetAttr, Option<TokenStream2>)> {
        if let Some(attrs) = attrs {
            let mut widget_attr = WidgetAttr::None;
            let mut doc_attr: Option<TokenStream2> = None;

            for attr in attrs.inner {
                match attr {
                    Attr::Local(_) => {
                        if widget_attr == WidgetAttr::None {
                            widget_attr = WidgetAttr::Local;
                        } else {
                            return Err(attr_twice_error(&attr));
                        }
                    }
                    Attr::LocalRef(_) => {
                        if widget_attr == WidgetAttr::None {
                            widget_attr = WidgetAttr::LocalRef;
                        } else {
                            return Err(attr_twice_error(&attr));
                        }
                    }
                    Attr::Doc(tokens) => {
                        if let Some(doc_tokens) = &mut doc_attr {
                            doc_tokens.extend(tokens);
                        } else {
                            doc_attr = Some(tokens);
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            attr.span(),
                            "Widgets can only have docs and `local` or `local_ref` as attribute.",
                        ));
                    }
                }
            }

            Ok((widget_attr, doc_attr))
        } else {
            Ok((WidgetAttr::None, None))
        }
    }

    // Make sure that the widget function is just a single identifier of the
    // local variable if a local attribute was set.
    fn local_attr_name(func: &WidgetFunc) -> Result<Ident> {
        let error_fn = |span| {
            Error::new(
                span,
                "Expected identifier due to the `local` or `local_ref` attribute.",
            )
        };

        match &func.path {
            WidgetFuncPath::Path(path) => {
                if let Some(name) = path.get_ident() {
                    Ok(name.clone())
                } else {
                    Err(error_fn(path.span()))
                }
            }
            WidgetFuncPath::Method(method) => Err(error_fn(method.span())),
        }
    }

    /// Parse information related to the widget function.
    fn parse_widget_func(input: ParseStream) -> Result<WidgetFuncInfo> {
        let inner_input: Option<ParseBuffer>;

        let upcoming_some = {
            let forked_input = input.fork();
            if forked_input.peek(Ident) {
                let ident: Ident = forked_input.parse()?;
                ident == "Some"
            } else {
                false
            }
        };

        let wrapper = if upcoming_some && input.peek2(token::Paren) {
            let ident = input.parse()?;
            let paren_input;
            parenthesized!(paren_input in input);
            inner_input = Some(paren_input);
            Some(ident)
        } else {
            inner_input = None;
            None
        };

        // get the inner input as func_input
        let func_input = if let Some(paren_input) = &inner_input {
            paren_input
        } else {
            input
        };

        // Look for &
        let ref_token = func_input.parse().ok();

        // Look for *
        let deref_token = func_input.parse().ok();

        let func: WidgetFunc = func_input.parse()?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        Ok((wrapper, ref_token, deref_token, func, properties))
    }
}
