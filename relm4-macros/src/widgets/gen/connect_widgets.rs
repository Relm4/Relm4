use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

use super::{util, Property, PropertyType};

impl PropertyType {
    fn connect_widgets_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Widget(widget) = self {
            Some(widget.widget_assignment())
        } else {
            None
        }
    }
}

impl Property {
    pub fn connect_widgets_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        if let Some(component_tokens) = self.ty.connect_widgets_tokens() {
            let args_stream = self.args_stream();
            let assign_fn = self.name.assign_fn_stream(&self.generics, w_name);
            let self_assign_args = self.name.assign_args_stream(w_name);

            assert!(self_assign_args.is_none());

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
