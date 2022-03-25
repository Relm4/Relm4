use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token, Error, Expr, ExprMacro, Ident, Macro, Result, Token,
};

use crate::widgets::{Property, PropertyName, PropertyType};

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
        else if (input.peek(Token![=])
            || input.peek3(Token![=])
            || (input.peek(Token![:]) && input.peek2(Token![mut]) && input.peek3(Ident)))
            // Don't interpret `property: value == other,` as a widget
            && !input.peek3(Token![==])
        {
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

            if input.peek2(Token![!]) && !input.peek3(Token![=]) {
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
