use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{bracketed, parenthesized, token, Error, Expr, Ident, Lit, LitStr, Path, Result, Token};

use crate::widgets::{Attr, Attrs};

impl Parse for Attrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut attrs = Vec::new();

        while input.peek(Token![#]) {
            let _sharp: Token![#] = input.parse()?;
            let attr_tokens;
            bracketed!(attr_tokens in input);
            let path: Path = attr_tokens.parse()?;

            // Name attribute
            attrs.push(if attr_tokens.is_empty() {
                if let Some(ident) = path.get_ident() {
                    if ident == "local" {
                        Attr::Local(ident.clone())
                    } else if ident == "local_ref" {
                        Attr::LocalRef(ident.clone())
                    } else if ident == "root" {
                        Attr::Root(ident.clone())
                    } else if ident == "watch" {
                        Attr::Watch(ident.clone())
                    } else if ident == "track" {
                        Attr::Track(ident.clone(), None)
                    } else if ident == "iterate" {
                        Attr::Iterate(ident.clone())
                    } else if ident == "template" {
                        Attr::Template(ident.clone())
                    } else if ident == "template_child" {
                        Attr::TemplateChild(ident.clone())
                    } else {
                        #[cfg(feature = "format")]
                        if ident == "BLANK" {
                            Attr::BlankLine
                        } else {
                            return Err(unexpected_attr_name(ident));
                        }

                        #[cfg(not(feature = "format"))]
                        return Err(unexpected_attr_name(ident));
                    }
                } else {
                    return Err(Error::new(path.span(), "Expected identifier."));
                }

            // List attribute: `#[name(item1, item2)]
            } else if attr_tokens.peek(token::Paren) {
                let paren_input;
                parenthesized!(paren_input in attr_tokens);
                let nested: Punctuated<Expr, token::Comma> =
                    Punctuated::parse_terminated(&paren_input)?;

                if let Some(ident) = path.get_ident() {
                    if ident == "block_signal" {
                        let mut signal_idents = Vec::with_capacity(nested.len());
                        for expr in nested {
                            let ident = expect_ident_from_expr(&expr)?;
                            signal_idents.push(ident);
                        }
                        Attr::BlockSignal(ident.clone(), signal_idents)
                    } else if ident == "track" {
                        let expr = expect_one_nested_expr(&nested)?;
                        Attr::Track(ident.clone(), Some(Box::new(expr.clone())))
                    } else if ident == "transition" {
                        let expr = expect_one_nested_expr(&nested)?;
                        let ident = expect_ident_from_expr(expr)?;
                        Attr::Transition(ident.clone(), ident)
                    } else if ident == "name" {
                        let expr = expect_one_nested_expr(&nested)?;
                        let ident = expect_ident_from_expr(expr)?;
                        Attr::Name(ident.clone(), ident)
                    } else if ident == "wrap" {
                        let expr = expect_one_nested_expr(&nested)?;
                        let path = expect_path_from_expr(expr)?;
                        Attr::Wrap(ident.clone(), path)
                    } else if ident == "chain" {
                        let expr = expect_one_nested_expr(&nested)?;
                        Attr::Chain(ident.clone(), Box::new(expr.clone()))
                    } else {
                        return Err(unexpected_attr_name(ident));
                    }
                } else {
                    return Err(Error::new(path.span(), "Expected identifier."));
                }

            // Value attribute: `#[name = literal)]
            } else if attr_tokens.peek(Token![=]) {
                let _eq: Token![=] = attr_tokens.parse()?;
                let lit = attr_tokens.parse()?;

                if let Some(ident) = path.get_ident() {
                    if ident == "track" {
                        let string = expect_string_lit(&lit)?;
                        Attr::Track(ident.clone(), Some(string.parse()?))
                    } else if ident == "doc" {
                        Attr::Doc(lit.into_token_stream())
                    } else if ident == "transition" {
                        let string = expect_string_lit(&lit)?;
                        Attr::Transition(ident.clone(), string.parse()?)
                    } else if ident == "name" {
                        let string = expect_string_lit(&lit)?;
                        Attr::Name(ident.clone(), string.parse()?)
                    } else {
                        return Err(unexpected_attr_name(ident));
                    }
                } else {
                    return Err(Error::new(path.span(), "Expected identifier."));
                }
            } else {
                return Err(Error::new(attr_tokens.span(), "Expected `]`, `(` or `=`."));
            });
        }

        Ok(Attrs { inner: attrs })
    }
}

fn unexpected_attr_name(ident: &Ident) -> Error {
    Error::new(
        ident.span(),
        format!("Unexpected attribute name `{ident}`."),
    )
}

fn expect_string_lit(lit: &Lit) -> Result<&LitStr> {
    if let Lit::Str(string) = lit {
        Ok(string)
    } else {
        Err(Error::new(
            lit.span(),
            "Expected string literal. Try this: `\"value\"`.",
        ))
    }
}

fn expect_path_from_expr(expr: &Expr) -> Result<Path> {
    if let Expr::Path(path) = expr {
        Ok(path.path.clone())
    } else {
        Err(Error::new(
            expr.span(),
            format!("Expected path `{}`.", expr.to_token_stream()),
        ))
    }
}

fn expect_ident_from_expr(expr: &Expr) -> Result<Ident> {
    if let Expr::Path(path) = expr {
        expect_ident_from_path(&path.path)
    } else {
        Err(Error::new(
            expr.span(),
            format!("Expected identifier `{}`.", expr.to_token_stream()),
        ))
    }
}

fn expect_ident_from_path(path: &Path) -> Result<Ident> {
    if let Some(ident) = path.get_ident() {
        Ok(ident.clone())
    } else {
        Err(Error::new(path.span(), "Expected identifier."))
    }
}

fn expect_one_nested_expr(nested: &Punctuated<Expr, token::Comma>) -> Result<&Expr> {
    if nested.len() == 1 {
        Ok(nested.first().unwrap())
    } else {
        Err(Error::new(nested.span(), "Expected only one expression."))
    }
}
