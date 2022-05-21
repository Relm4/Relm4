use proc_macro2::{Literal, Punct};
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::{And, At, Caret, Colon, Div, Dot, Gt, Lt, Or, Question, Tilde, Underscore};
use syn::{braced, bracketed, parenthesized, token, Ident, Lifetime, Token};

use crate::widgets::{parse_util, ParseError, Properties, Property, PropertyName, PropertyType};

impl Properties {
    pub(super) fn parse(input: ParseStream) -> Self {
        let mut props: Punctuated<Property, Token![,]> = Punctuated::new();
        loop {
            if input.is_empty() {
                break;
            }
            let parse_input = input.fork();
            let (prop, contains_error) = Property::parse(&parse_input);
            props.push(prop);

            // Everything worked, advance input
            if !contains_error {
                input.advance_to(&parse_input);
            }

            if input.is_empty() {
                break;
            }

            if let Err(prop) = parse_comma_error(input) {
                // If there's already an error, ignore the additional comma error
                if !contains_error {
                    props.push(prop);
                } else {
                    // Skip to next token to start with "fresh" and hopefully correct syntax.
                    while !parse_next_token(input).unwrap() {
                        let next_input = input.fork();
                        let (prop, contains_error) = Property::parse(&next_input);
                        if !contains_error {
                            // Point with correct syntax was found!
                            props.push(prop);
                            input.advance_to(&next_input);

                            // Now we should definitely have a comma
                            if let Err(prop) = parse_comma_error(input) {
                                props.push(prop);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let properties = props.into_pairs().map(|pair| pair.into_value()).collect();
        Properties { properties }
    }
}

fn parse_comma_error(input: ParseStream) -> Result<(), Property> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![,]) {
        input.parse::<Token![,]>().unwrap();
        Ok(())
    } else {
        let err = lookahead.error();
        Err(Property {
            name: PropertyName::Ident(parse_util::string_to_snake_case("comma_error")),
            ty: PropertyType::ParseError(ParseError::Generic(err.to_compile_error())),
        })
    }
}

macro_rules! parse_type {
    ($input:ident, $ty:ty) => {
        let _: $ty = $input.parse()?;
        return Ok(false);
    };
}

fn parse_next_token(input: ParseStream) -> Result<bool, syn::Error> {
    let _unused;
    if input.is_empty() {
        Ok(true)
    } else if input.peek(Token![,]) {
        let _comma: Token![,] = input.parse()?;
        Ok(true)
    } else if input.peek(token::Paren) {
        parenthesized!(_unused in input);
        Ok(false)
    } else if input.peek(token::Bracket) {
        bracketed!(_unused in input);
        Ok(false)
    } else if input.peek(token::Brace) {
        braced!(_unused in input);
        Ok(false)
    } else if input.peek(Ident) {
        let _ident = Ident::parse_any(input)?;
        Ok(false)
    } else if input.peek(And) {
        parse_type!(input, And);
    } else if input.peek(At) {
        parse_type!(input, At);
    } else if input.peek(Colon) {
        parse_type!(input, Colon);
    } else if input.peek(Div) {
        parse_type!(input, Div);
    } else if input.peek(syn::token::Eq) {
        parse_type!(input, syn::token::Eq);
    } else if input.peek(Gt) {
        parse_type!(input, Gt);
    } else if input.peek(Lt) {
        parse_type!(input, Lt);
    } else if input.peek(Or) {
        parse_type!(input, Or);
    } else if input.peek(Tilde) {
        parse_type!(input, Tilde);
    } else if input.peek(Caret) {
        parse_type!(input, Caret);
    } else if input.peek(Underscore) {
        parse_type!(input, Underscore);
    } else if input.peek(Question) {
        parse_type!(input, Question);
    } else if input.peek(Dot) {
        parse_type!(input, Dot);
    } else if input.peek(Lifetime) {
        parse_type!(input, Lifetime);
    } else if input.parse::<Punct>().is_ok() || input.parse::<Literal>().is_ok() {
        Ok(false)
    } else {
        unreachable!("Every possible token should be covered. Please report this error at Relm4!");
    }
}
