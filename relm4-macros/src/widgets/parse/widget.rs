use syn::{
    braced, parenthesized,
    parse::{ParseBuffer, ParseStream},
    spanned::Spanned,
    token,
    token::{And, Star},
    Error, Expr, Ident, Result, Token,
};

use crate::widgets::{util::attr_twice_error, Attr, Attrs, Properties, Widget, WidgetFunc};
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
        let attr = Self::process_attributes(attributes)?;
        // Check if first token is `mut`
        let mutable = input.parse().ok();

        // Look for name = Widget syntax
        let name_opt: Option<Ident> = if input.peek2(Token![=]) {
            let name_opt = Some(input.parse()?);
            let _token: Token![=] = input.parse()?;
            name_opt
        } else {
            None
        };

        let (wrapper, ref_token, deref_token, func, properties) = Self::parse_widget_func(input)?;

        // Generate a name if no name was given.
        let name = if let Some(name) = name_opt {
            name
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
        let attr = Self::process_attributes(attributes)?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        // Generate a name
        let name = func.snake_case_name();

        let ref_token = Some(And::default());

        Ok(Widget {
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

    fn process_attributes(attrs: Option<Attrs>) -> Result<WidgetAttr> {
        if let Some(attrs) = attrs {
            let mut local = false;

            for attr in attrs.inner {
                if let Attr::Local(_) = attr {
                    if local {
                        return Err(attr_twice_error(&attr));
                    } else {
                        local = true;
                    }
                } else {
                    return Err(Error::new(
                        attr.span(),
                        "Widgets can only have `local` or `local_ref` as attribute.",
                    ));
                }
            }

            Ok(if local {
                WidgetAttr::Local
            } else {
                WidgetAttr::None
            })
        } else {
            Ok(WidgetAttr::None)
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
