use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Path};

use super::{util, Property, PropertyType};

impl PropertyType {
    fn view_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Watch(token_stream) => Some(token_stream.clone()),
            _ => None,
        }
    }
}

impl Property {
    pub fn view_stream(&self, stream: &mut TokenStream2, w_name: &Ident, relm4_path: &Path, widgets_as_self: bool) {
        if let Some(p_assign) = self.ty.view_assign_tokens() {
            let assign_fn = self.name.self_assign_fn_stream(&self.generics, w_name, widgets_as_self);
            let self_assign_args = self.name.self_assign_args_stream(w_name, widgets_as_self);

            util::property_assign_tokens(
                stream,
                self,
                assign_fn,
                self_assign_args,
                p_assign,
                None,
                None,
            );
        }

        let fact_assign_opt = self.ty.factory_expr();
        if let Some(f_expr) = fact_assign_opt {
        let self_token = if widgets_as_self {
            quote! { widgets }
        } else {
            quote! { self }
        };

            stream.extend(quote! {
                #relm4_path::factory::Factory::generate(&#f_expr, & #self_token.#w_name, sender.clone());
            });
        }
    }
}
