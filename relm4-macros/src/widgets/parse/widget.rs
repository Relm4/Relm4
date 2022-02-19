use syn::{
    braced, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    token,
    token::{And, Star},
    Ident, Result, Token,
};

use crate::util;
use crate::widgets::{Properties, Widget, WidgetFunc};

type WidgetFuncInfo = (
    Option<Ident>,
    Option<And>,
    Option<Star>,
    WidgetFunc,
    Properties,
);

/// Parse information related to the widget function.
fn parse_widget_func(input: ParseStream) -> Result<WidgetFuncInfo> {
    let inner_input: Option<ParseBuffer>;

    let upcomming_some = {
        let forked_input = input.fork();
        if forked_input.peek(Ident) {
            let ident: Ident = forked_input.parse()?;
            ident == "Some"
        } else {
            false
        }
    };

    let wrapper = if upcomming_some && input.peek2(token::Paren) {
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
        &paren_input
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

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_opt: Option<Ident> = None;

        // Check if first token is `mut`
        let mutable = input.parse().ok();

        // Look for name = Widget syntax
        if input.peek2(Token![=]) {
            name_opt = Some(input.parse()?);
            let _token: Token![=] = input.parse()?;
        };

        let (wrapper, ref_token, deref_token, func, properties) = parse_widget_func(input)?;

        // Generat a name if no name was given.
        let name = if let Some(name) = name_opt {
            name
        } else {
            util::idents_to_snake_case(&func.path_segments, func.span)
        };

        let returned_widget = if input.peek(Token![->]) {
            let _arrow: Token![->] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Widget {
            mutable,
            name,
            func,
            properties,
            wrapper,
            ref_token,
            deref_token,
            returned_widget,
        })
    }
}

impl Widget {
    pub fn parse_for_container_ext(input: ParseStream) -> Result<Self> {
        // Look for &
        let ref_token = input.parse().ok();

        // Look for *
        let deref_token = input.parse().ok();

        let func: WidgetFunc = input.parse()?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        // Generat a name
        let name = util::idents_to_snake_case(&func.path_segments, func.span);

        Ok(Widget {
            mutable: None,
            name,
            func,
            properties,
            wrapper: None,
            ref_token,
            deref_token,
            returned_widget: None,
        })
    }
}
