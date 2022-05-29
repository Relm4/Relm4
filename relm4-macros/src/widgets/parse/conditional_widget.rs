use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{Error, Expr, Ident, Token};

use crate::args::Args;
use crate::widgets::parse_util::{self, attr_twice_error};
use crate::widgets::{Attr, Attrs, ConditionalBranches, ConditionalWidget, ParseError};

type ConditionalAttrs = (Option<Ident>, Option<Ident>, Option<TokenStream2>);

impl ConditionalWidget {
    pub(super) fn parse(
        input: ParseStream,
        attrs: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self, ParseError> {
        let name = if input.peek2(Token![=]) && !input.peek2(Token![==]) && !input.peek(Token![!]) {
            let name = input.parse()?;
            let _assign: Token![=] = input.parse()?;
            Some(name)
        } else {
            None
        };
        Self::parse_with_name(input, name, attrs, args)
    }

    pub(super) fn parse_with_name(
        input: ParseStream,
        name: Option<Ident>,
        attrs: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self, ParseError> {
        let (transition, attr_name, doc_attr) = Self::process_attrs(attrs)?;

        if attr_name.is_some() {
            if let Some(name) = &name {
                return Err(Error::new(
                    name.span(),
                    "Name defined as attribute and redefined here.",
                )
                .into());
            }
        }

        let name = if let Some(name) = name {
            name
        } else if let Some(name) = attr_name {
            name
        } else {
            parse_util::idents_to_snake_case(
                [Ident::new("conditional_widget", input.span())].iter(),
                input.span(),
            )
        };

        if input.peek(Token![if]) {
            let branches = ConditionalBranches::parse_if(input)?;
            Ok(Self {
                doc_attr,
                name,
                transition,
                branches,
                args,
            })
        } else if input.peek(Token![match]) {
            let branches = ConditionalBranches::parse_match(input)?;
            Ok(Self {
                name,
                transition,
                branches,
                args,
                doc_attr,
            })
        } else {
            Err(input.error("Expected `if` or `match`").into())
        }
    }

    fn process_attrs(attrs: Option<Attrs>) -> Result<ConditionalAttrs, ParseError> {
        let mut transition = None;
        let mut name = None;
        let mut doc_attr: Option<TokenStream2> = None;
        if let Some(attrs) = attrs {
            for attr in attrs.inner {
                match attr {
                    Attr::Transition(_, ref transition_value) => {
                        if transition.is_none() {
                            transition = Some(transition_value.clone());
                        } else {
                            return Err(attr_twice_error(&attr).into());
                        }
                    }
                    Attr::Name(_, ref name_value) => {
                        if name.is_none() {
                            name = Some(name_value.clone());
                        } else {
                            return Err(attr_twice_error(&attr).into());
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
                            "Conditional widgets can only have docs and `name` or `transition` as attribute.",
                        ).into());
                    }
                }
            }
        }
        Ok((transition, name, doc_attr))
    }
}
