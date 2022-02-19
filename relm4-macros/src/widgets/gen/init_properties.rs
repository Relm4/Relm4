use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Ident, Path};

use super::{util, Property, PropertyType, Tracker};

impl PropertyType {
    fn init_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Expr(expr) => Some(expr.to_token_stream()),
            PropertyType::Watch(tokens) => Some(tokens.to_token_stream()),
            PropertyType::Args(args) => Some(args.to_token_stream()),
            PropertyType::Track(Tracker {
                bool_fn,
                update_fns,
            }) => Some(if update_fns.is_empty() {
                quote! { #bool_fn }
            } else {
                quote! { #(#update_fns),* }
            }),
            _ => None,
        }
    }
}

impl Property {
    pub fn property_init_stream(
        &self,
        stream: &mut TokenStream2,
        parent_name: &Ident,
        relm4_path: &Path,
    ) {
        if let Some(p_assign) = self.ty.init_assign_tokens() {
            let args_stream = self.args_stream();

            let assign_fn = self
                .name
                .assign_fn_stream(&self.generics, parent_name, relm4_path);
            let self_assign_args = self.name.assign_args_stream(parent_name);

            util::property_assign_tokens(
                stream,
                self,
                assign_fn,
                self_assign_args,
                p_assign,
                None,
                args_stream,
            );
        }

        let fact_assign_opt = self.ty.factory_expr();
        if let Some(f_expr) = fact_assign_opt {
            stream.extend(quote! {
                #relm4_path::factory::Factory::generate(&#f_expr, &#parent_name, sender.clone());
            });
        }
    }
}
