use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, quote, ToTokens};
use syn::{spanned::Spanned, Expr, ExprPath, Ident};

use super::{util, Property, PropertyType};

impl PropertyType {
    fn component_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Widget(widget) => Some(widget.widget_assignment()),
            PropertyType::Component(expr) => Some(component_tokens(expr)),
            _ => None,
        }
    }
}

impl Property {
    pub fn component_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        if let Some(component_tokens) = self.ty.component_tokens() {
            let args_stream = self.args_stream();
            let assign_fn = self.name.self_assign_fn_stream(&self.generics, w_name);
            let self_assign_args = self.name.self_assign_args_stream(w_name);

            let mut inner_stream = TokenStream2::new();
            util::property_assign_tokens(
                &mut inner_stream,
                self,
                assign_fn,
                self_assign_args,
                component_tokens,
                None,
                Some(args_stream),
            );

            if let Some(return_stream) = self.ty.return_assign_tokens() {
                stream.extend(quote! {
                    #return_stream = #inner_stream;
                })
            } else {
                stream.extend(inner_stream);
            }
        }
    }
}

fn component_ident(path: &ExprPath) -> TokenStream2 {
    if path.path.segments.len() == 1 {
        let ident = &path.path.segments.first().unwrap().ident;
        quote_spanned! { path.span() => components.#ident.root_widget() }
    } else {
        path.to_token_stream()
    }
}

fn component_tokens(expr: &Expr) -> TokenStream2 {
    match expr {
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(segs) = path.path.segments.first() {
                    if segs.ident == "Some" {
                        if call.args.len() == 1 {
                            if let Expr::Path(args_path) = call.args.first().unwrap() {
                                let arg_tokens = component_ident(args_path);
                                quote_spanned! { path.span() => Some(#arg_tokens) }
                            } else {
                                expr.to_token_stream()
                            }
                        } else {
                            expr.to_token_stream()
                        }
                    } else {
                        expr.to_token_stream()
                    }
                } else {
                    expr.to_token_stream()
                }
            } else {
                expr.to_token_stream()
            }
        }
        Expr::Path(path) => component_ident(path),
        _ => expr.to_token_stream(),
    }
}
