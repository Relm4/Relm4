/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Expr, ExprMacro, Ident, Lit, Macro, Result, Token,
};

use crate::util;

use super::{Properties, Property, PropertyType, Tracker, Widget, WidgetFunc};

impl Parse for Tracker {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![input.parse()?];

        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            items.push(input.parse()?);
        }

        let update_fn = if let Some(item) = items.pop() {
            Ok(item)
        } else {
            Err(input.error("Expected identifier or expression"))
        }?;

        if items.is_empty() {
            return Err(input.error("Expected at least two arguments"));
        }

        Ok(Tracker { items, update_fn })
    }
}

/*impl Parse for Factory {
    fn parse(input: ParseStream) -> Result<Self> {
        let data = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let widget_name = input.parse()?;

        Ok(Factory { data, widget_name })
    }
}*/

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let mut optional_assign = false;
        let mut iterative = false;

        if input.peek(Token![!]) {
            return if name == "factory" {
                let _exclm: Token![!] = input.parse()?;
                let paren_input;
                parenthesized!(paren_input in input);
                Ok(Property {
                    name,
                    ty: PropertyType::Factory(paren_input.parse()?),
                    optional_assign,
                    iterative,
                    args: None,
                })
            } else {
                Err(input.error("Expected macro factory"))
            };
        }

        // check for property(a, b, c): ...
        let args = if input.peek(token::Paren) {
            let paren_input;
            parenthesized!(paren_input in input);
            Some(paren_input.parse()?)
        } else {
            None
        };

        let ty = if input.peek(Token! [=>]) {
            let _arrow: Token![=>] = input.parse()?;
            input.parse().map(PropertyType::Connect)?
        } else if input.peek(Token![=]) || input.peek3(Token![=]) {
            if input.peek(Token![=]) {
                let _token: Token![=] = input.parse()?;
            } else {
                let _colon: Token! [:] = input.parse()?;
            }
            input.parse().map(PropertyType::Widget)?
        } else if input.peek(Token! [:]) || input.peek(Token! [?]) {
            // look for ? at beginning for optional assign
            if input.peek(Token! [?]) {
                let _question_mark: Token![?] = input.parse()?;
                optional_assign = true;
            }
            let _colon: Token! [:] = input.parse()?;
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
                    } else if ident == "component" {
                        let tokens = mac.tokens.into();
                        PropertyType::Component(parse_macro_input::parse(tokens)?)
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
                input.parse().map(PropertyType::Expr)?
            }
        } else {
            return Err(input.error("TODO"));
        };

        Ok(Property {
            name,
            ty,
            args,
            optional_assign,
            iterative,
        })
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

        if input.peek2(Token![=]) {
            name_opt = Some(input.parse()?);
            let _token: Token![=] = input.parse()?;
        };

        let inner_input: Option<ParseBuffer>;

        let wrapper = if input.peek(Ident) && input.peek2(token::Paren) {
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

        Ok(Widget {
            name,
            func,
            properties,
            wrapper,
            assign_as_ref,
        })
    }
}
