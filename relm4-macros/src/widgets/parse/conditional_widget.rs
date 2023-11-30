use proc_macro2::TokenStream as TokenStream2;
use syn::parse::ParseStream;
use syn::{Error, Expr, Ident, Path, Token};

use crate::args::Args;
use crate::widgets::parse_util::{self, attr_twice_error};
use crate::widgets::{Attr, Attrs, ConditionalBranches, ConditionalWidget, ParseError};

type ConditionalAttrs = (
    Option<Ident>,
    Option<Ident>,
    Option<TokenStream2>,
    Option<Path>,
);

impl ConditionalWidget {
    pub(super) fn parse(
        input: ParseStream<'_>,
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
        input: ParseStream<'_>,
        name: Option<Ident>,
        attrs: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self, ParseError> {
        let (transition, attr_name, doc_attr, assign_wrapper) = Self::process_attrs(attrs)?;

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
            parse_util::unique_ident_from_parts(["conditional_widget"])
        };

        if input.peek(Token![if]) {
            let branches = ConditionalBranches::parse_if(input)?;
            Ok(Self {
                doc_attr,
                transition,
                assign_wrapper,
                name,
                args,
                branches,
            })
        } else if input.peek(Token![match]) {
            let branches = ConditionalBranches::parse_match(input)?;
            Ok(Self {
                doc_attr,
                transition,
                assign_wrapper,
                name,
                args,
                branches,
            })
        } else {
            Err(input.error("Expected `if` or `match`").into())
        }
    }

    fn process_attrs(attrs: Option<Attrs>) -> Result<ConditionalAttrs, ParseError> {
        let mut transition = None;
        let mut name = None;
        let mut doc_attr: Option<TokenStream2> = None;
        let mut assign_wrapper = None;

        if let Some(attrs) = attrs {
            for attr in attrs.inner {
                let span = attr.span();
                match attr {
                    Attr::Transition(_, transition_value) => {
                        if transition.is_none() {
                            transition = Some(transition_value);
                        } else {
                            return Err(attr_twice_error(span).into());
                        }
                    }
                    Attr::Name(_, name_value) => {
                        if name.is_none() {
                            name = Some(name_value);
                        } else {
                            return Err(attr_twice_error(span).into());
                        }
                    }
                    Attr::Wrap(_, path) => {
                        if assign_wrapper.is_some() {
                            return Err(attr_twice_error(span).into());
                        }
                        assign_wrapper = Some(path.clone());
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
        Ok((transition, name, doc_attr, assign_wrapper))
    }
}
