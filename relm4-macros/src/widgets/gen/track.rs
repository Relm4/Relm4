use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Error, Expr, ExprCall, ExprField, Ident, Member, Type};

use super::{Property, PropertyType, Tracker};

impl PropertyType {
    fn track_tokens(&self, model_type: &Type) -> Option<(TokenStream2, TokenStream2)> {
        if let PropertyType::Track(Tracker {
            update_fns,
            bool_fn,
        }) = self
        {
            // Only one parameter passed. Try to generate tracker.
            if update_fns.is_empty() {
                let bool_stream = match generate_tracker_from_expression(bool_fn, model_type) {
                    Ok(bool_tokens) => bool_tokens,
                    Err((span, msg)) => {
                        return Some((
                            Error::new(span, &msg).to_compile_error(),
                            TokenStream2::new(),
                        ));
                    }
                };
                Some((bool_stream, bool_fn.to_token_stream()))
            } else {
                let update_stream = quote! { #(#update_fns),* };
                let bool_stream = bool_fn.to_token_stream();

                // TODO: Uncomment this and add a warning once proc-macro warnings are stable
                /*if update_fns.len() == 1 {
                    if let Ok(auto_bool_stream) =
                        generate_tracker_from_expression(&update_fns[0], model_name)
                    {
                        if auto_bool_stream.to_string() == bool_stream.to_string() {
                            let error_msg = "Consider removing the first parameter because the macro would generate the same code.\n";
                            /*println!(Some((
                                Error::new(bool_fn.span(), error_msg).to_compile_error(),
                                update_stream,
                            ));*/
                        }
                    }
                }*/

                Some((bool_stream, update_stream))
            }
        } else {
            None
        }
    }
}

impl Property {
    pub fn track_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        model_type: &Type,
        self_as_widgets: bool,
    ) {
        if let Some((bool_stream, update_stream)) = self.ty.track_tokens(model_type) {
            let p_name = &self.name;
            let p_span = p_name.span().unwrap().into();

            let assign_fn =
                self.name
                    .self_assign_fn_stream(&self.generics, w_name, self_as_widgets);
            let self_assign_args = self.name.self_assign_args_stream(w_name, self_as_widgets);
            let args_stream = self.args_stream();

            stream.extend(quote_spanned! {
                p_span =>  if #bool_stream {
                    #assign_fn(#self_assign_args #update_stream #args_stream);
            }});
        }
    }
}

/// Helper function for the tracker macro.
fn expr_field_from_expr_call(call_expr: &ExprCall) -> Option<&ExprField> {
    let first_expr = call_expr.args.iter().next()?;
    if let Expr::Field(expr_field) = first_expr {
        Some(expr_field)
    } else {
        None
    }
}

fn generate_tracker_from_expression(
    expression: &Expr,
    model_name: &Type,
) -> Result<TokenStream2, (Span2, String)> {
    let error_fn = move |span, msg: &str| {
        let error_msg =
                    "Unable to generate tracker function. Please pass a tracker function as the first parameter to the `track!` macro.\n\
                    Usage: track!(TRACK_CONDITION: bool, FIRST_ARG, SECOND_ARG, ...)";
        Err((span, format!("{}\nHint:  {}", error_msg, msg)))
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

    let base_is_model = if let Expr::Path(expr_path) = &*expr_field.base {
        if let Some(ident) = expr_path.path.get_ident() {
            ident == "model"
        } else {
            false
        }
    } else {
        false
    };

    if !base_is_model {
        return error_fn(
            expr_field.base.span(),
            "Couldn't find a reference to `model`.",
        );
    }

    let ident = if let Member::Named(ident) = &expr_field.member {
        ident.clone()
    } else {
        return error_fn(expr_field.member.span(), "Expected a named member");
    };

    let bool_stream =
        quote_spanned! { expr_field.span() => model.changed( #model_name::#ident() ) };
    Ok(bool_stream)
}
