use proc_macro2::Span as Span2;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Error, Expr, ExprMacro, Ident, Lit, Macro, Result, Token,
};

use crate::util;

use super::{
    Properties, Property, PropertyName, PropertyType, ReturnedWidget, Tracker, Widget, WidgetFunc,
};

impl Parse for Tracker {
    fn parse(input: ParseStream) -> Result<Self> {
        let bool_fn = input.parse()?;

        let mut update_fns = Vec::new();
        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            // allow comma at the end of the macro
            if !input.is_empty() {
                update_fns.push(input.parse()?);
            }
        }

        Ok(Tracker {
            bool_fn,
            update_fns,
        })
    }
}

impl Parse for PropertyName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(Token![::]) || input.peek2(Token! [::]) {
            PropertyName::Path(input.parse()?)
        } else {
            PropertyName::Ident(input.parse()?)
        })
    }
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let mut optional_assign = false;
        let mut iterative = false;
        let mut braced_args = false;

        if input.peek(Token![!]) {
            if let PropertyName::Ident(ref ident_name) = name {
                if ident_name == "factory" {
                    let _exclm: Token![!] = input.parse()?;
                    let paren_input;
                    parenthesized!(paren_input in input);
                    return Ok(Property {
                        name,
                        ty: PropertyType::Factory(paren_input.parse()?),
                        generics: None,
                        optional_assign,
                        iterative,
                        args: None,
                    });
                }
            }
            return Err(input.error("Expected factory macro"));
        }

        // check for property(a, b, c): ...
        let args = if input.peek(token::Paren) {
            let paren_input;
            parenthesized!(paren_input in input);
            Some(paren_input.parse()?)
        }
        // check for property[a, b, c]: ...
        else if input.peek(token::Bracket) {
            let paren_input;
            bracketed!(paren_input in input);
            braced_args = true;
            Some(paren_input.parse()?)
        } else {
            None
        };

        let generics = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };

        // look for event handlers: property(a, ...) => move |a, ...| { ... }
        let ty = if input.peek(Token! [=>]) {
            let _arrow: Token![=>] = input.parse()?;
            if braced_args {
                input.parse().map(PropertyType::ConnectComponent)?
            } else {
                input.parse().map(PropertyType::Connect)?
            }
        }
        // look for widgets
        else if input.peek(Token![=]) || input.peek3(Token![=]) || (input.peek(Token![:]) && input.peek2(Token![mut]) && input.peek3(Ident)) {
            if input.peek(Token![=]) {
                let _token: Token![=] = input.parse()?;
            } else {
                let _colon: Token![:] = input.parse()?;
            }
            input.parse().map(PropertyType::Widget)?
        }
        // look for properties or optional properties (?)
        else if input.peek(Token! [:]) || input.peek(Token! [?]) {
            // look for ? at beginning for optional assign
            if input.peek(Token! [?]) {
                let _question_mark: Token![?] = input.parse()?;
                optional_assign = true;
            }
            let colon: Token! [:] = input.parse()?;
            let colon_span = colon.span();

            if input.peek(Lit) {
                input.parse().map(PropertyType::Value)?
            } else if input.peek2(Token![!]) {
                let mac: Macro = input.parse()?;
                let segs = &mac.path.segments;

                if segs.len() == 1 {
                    let ident = &segs.first().expect("Macro has no segments").ident;

                    if ident == "track" {
                        let tokens = mac.tokens.into();
                        PropertyType::Track(parse_macro_input::parse(tokens)?)
                    } else if ident == "parent" {
                        let tokens = mac.tokens.into();
                        PropertyType::Parent(parse_macro_input::parse(tokens)?)
                    } else if ident == "args" {
                        let tokens = mac.tokens.into();
                        PropertyType::Args(parse_macro_input::parse(tokens)?)
                    } else if ident == "watch" {
                        PropertyType::Watch(mac.tokens)
                    } else if ident == "iterate" {
                        iterative = true;
                        let tokens = mac.tokens.into();
                        PropertyType::Expr(parse_macro_input::parse(tokens)?)
                    } else if ident == "iterate_watch" {
                        iterative = true;
                        let tokens = mac.tokens.into();
                        PropertyType::Watch(parse_macro_input::parse(tokens)?)
                    } else {
                        PropertyType::Expr(Expr::Macro(ExprMacro {
                            attrs: Vec::new(),
                            mac,
                        }))
                    }
                } else {
                    input.parse().map(PropertyType::Expr)?
                }
            } else {
                match input.parse().map(PropertyType::Expr) {
                    Ok(expr) => expr,
                    Err(parse_err) => {
                        let mut err = Error::new(colon_span, "Did you confuse `=` with`:`?");
                        err.combine(parse_err);
                        return Err(err);
                    }
                }
            }
        } else {
            return Err(input.error("Unexpected token. Expected =>, =, : or ?:"));
        };

        if !input.is_empty() && !input.peek(Token![,]) {
            Err(input.error("expected `,`. Did you confuse `=` with`:`?"))
        } else {
            Ok(Property {
                name,
                ty,
                generics,
                args,
                optional_assign,
                iterative,
            })
        }
    }
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        let props: Punctuated<Property, Token![,]> = input.parse_terminated(Property::parse)?;
        let properties = props.into_pairs().map(|pair| pair.into_value()).collect();
        Ok(Properties { properties })
    }
}

impl Parse for WidgetFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut path_segments = Vec::new();
        let mut args = None;
        let mut ty = None;

        path_segments.push(input.parse()?);

        loop {
            if input.peek(Ident) {
                path_segments.push(input.parse()?);
            } else if input.peek(Token! [::]) {
                let _colon: Token![::] = input.parse()?;
            } else if input.peek(token::Paren) {
                let paren_input;
                parenthesized!(paren_input in input);
                args = Some(paren_input.call(Punctuated::parse_terminated)?);
                if input.peek(Token! [->]) {
                    let _token: Token! [->] = input.parse()?;
                    let mut ty_path = vec![input.parse()?];

                    loop {
                        if input.peek(Ident) {
                            ty_path.push(input.parse()?);
                        } else if input.peek(Token! [::]) {
                            let _colon: Token![::] = input.parse()?;
                        } else {
                            break;
                        }
                    }
                    ty = Some(ty_path);
                }
                break;
            } else {
                break;
            }
        }

        Ok(WidgetFunc {
            path_segments,
            args,
            ty,
        })
    }
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

        let func_input = if let Some(paren_input) = &inner_input {
            &paren_input
        } else {
            input
        };

        let assign_as_ref = if func_input.peek(Token![&]) {
            let _ref: Token![&] = func_input.parse()?;
            true
        } else {
            false
        };

        let func: WidgetFunc = func_input.parse()?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        let name = if let Some(name) = name_opt {
            name
        } else {
            util::idents_to_snake_case(&func.path_segments)
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
            assign_as_ref,
            returned_widget,
        })
    }
}

impl Parse for ReturnedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut is_optional = false;

        let (name, ty) = if input.peek(Ident) {
            let name = input.parse()?;

            let _colon: Token![:] = input.parse()?;
            let ty = input.parse()?;

            if input.peek(Token![?]) {
                let _mark: Token![?] = input.parse()?;
                is_optional = true;
            }

            (Some(name), Some(ty))
        } else {
            if input.peek(Token![?]) {
                let _mark: Token![?] = input.parse()?;
                is_optional = true;
            }

            (None, None)
        };

        let name = name.unwrap_or_else(|| {
            crate::util::idents_to_snake_case(&[Ident::new("_returned_widget", Span2::call_site())])
        });

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        Ok(ReturnedWidget {
            name,
            ty,
            properties,
            is_optional,
        })
    }
}
