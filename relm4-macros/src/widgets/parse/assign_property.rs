use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{Error, Expr, ExprCall, ExprField, Ident, Member, Result, Token};

use crate::args::Args;
use crate::widgets::util::attr_twice_error;
use crate::widgets::{AssignProperty, AssignPropertyAttr, Attr, Attrs};

struct ProcessedAttrs {
    watch: AssignPropertyAttr,
    iterative: bool,
    block_signals: Vec<Ident>,
}

impl AssignProperty {
    pub(super) fn parse(
        input: ParseStream,
        attributes: Option<Attrs>,
        args: Option<Args<Expr>>,
    ) -> Result<Self> {
        let optional_assign = input.parse::<Token![?]>().is_ok();
        let colon: Token! [:] = input.parse()?;
        let colon_span = colon.span();

        let expr = match input.parse() {
            Ok(expr) => expr,
            Err(parse_err) => {
                let mut err = Error::new(colon_span, "Did you confuse `=` with`:`?");
                err.combine(parse_err);
                return Err(err);
            }
        };

        let ProcessedAttrs {
            watch,
            iterative,
            block_signals,
        } = Self::process_attributes(&expr, attributes)?;

        Ok(Self {
            attr: watch,
            expr,
            args,
            optional_assign,
            iterative,
            block_signals,
        })
    }

    fn process_attributes(assign_expr: &Expr, attrs: Option<Attrs>) -> Result<ProcessedAttrs> {
        if let Some(attrs) = attrs {
            let mut iterative = false;
            let mut watch = AssignPropertyAttr::None;
            let mut block_signals = Vec::with_capacity(0);

            for attr in attrs.inner {
                match attr {
                    Attr::Iterate(_) => {
                        if iterative {
                            return Err(attr_twice_error(&attr));
                        } else {
                            iterative = true;
                        }
                    }
                    Attr::Watch(_) => {
                        if watch == AssignPropertyAttr::None {
                            watch = AssignPropertyAttr::Watch;
                        } else {
                            return Err(attr_twice_error(&attr));
                        }
                    }
                    Attr::Track(_, ref expr) => {
                        if watch == AssignPropertyAttr::None {
                            watch = if let Some(expr) = expr {
                                AssignPropertyAttr::Track(expr.to_token_stream())
                            } else {
                                AssignPropertyAttr::Track(generate_tracker_from_expression(
                                    assign_expr,
                                )?)
                            };
                        } else {
                            return Err(attr_twice_error(&attr));
                        }
                    }
                    Attr::BlockSignal(name, idents) => {
                        if block_signals.is_empty() {
                            block_signals = idents;
                        } else {
                            return Err(attr_twice_error(&name));
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            attr.span(),
                            "Properties can only have `watch`, `track` or `iterative` as attribute.",
                        ));
                    }
                }
            }
            Ok(ProcessedAttrs {
                watch,
                iterative,
                block_signals,
            })
        } else {
            Ok(ProcessedAttrs {
                watch: AssignPropertyAttr::None,
                iterative: false,
                block_signals: Vec::with_capacity(0),
            })
        }
    }
}

/// Helper function for the tracker attribute.
fn expr_field_from_expr_call(call_expr: &ExprCall) -> Option<&ExprField> {
    let first_expr = call_expr.args.iter().next()?;
    if let Expr::Field(expr_field) = first_expr {
        Some(expr_field)
    } else {
        None
    }
}

fn generate_tracker_from_expression(expression: &Expr) -> Result<TokenStream2> {
    let error_fn = move |span, msg: &str| {
        let error_msg =
                    "Unable to generate tracker function. Please pass a tracker function as the first parameter to the `track!` macro.\n\
                    Usage: track!(TRACK_CONDITION: bool, FIRST_ARG, SECOND_ARG, ...)";
        Err(Error::new(span, format!("{}\nHint:  {}", error_msg, msg)))
    };

    let unref_expr: &Expr = if let Expr::Reference(expr_ref) = expression {
        &expr_ref.expr
    } else {
        expression
    };

    let expr_field_opt = match unref_expr {
        Expr::Call(call_expr) => expr_field_from_expr_call(call_expr),
        Expr::MethodCall(expr_method_call) => {
            if let Expr::Field(ref expr_field) = *expr_method_call.receiver {
                Some(expr_field)
            } else {
                None
            }
        }
        _ => None,
    };

    let expr_field = if let Some(expr_field) = expr_field_opt {
        expr_field
    } else {
        return error_fn(
            unref_expr.span(),
            "Couldn't find find a call or method expression.",
        );
    };

    let ident = if let Member::Named(ident) = &expr_field.member {
        ident.clone()
    } else {
        return error_fn(expr_field.member.span(), "Expected a named member");
    };

    let bool_stream =
        quote_spanned! { expr_field.span() => model.changed( __ModelType::#ident() ) };
    Ok(bool_stream)
}
