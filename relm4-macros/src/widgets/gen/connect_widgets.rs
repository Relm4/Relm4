use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

use super::{util, Property, PropertyType, ReturnedWidget};

impl PropertyType {
    pub fn connect_widget_with_unwrap(&self) -> bool {
        if let PropertyType::Widget(widget) = &self {
            if let Some(returned_widget) = &widget.returned_widget {
                returned_widget.is_optional
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl ReturnedWidget {
    pub fn return_assign_tokens(&self) -> TokenStream2 {
        let name = &self.name;

        if let Some(ty) = &self.ty {
            quote! {
                let #name : #ty
            }
        } else {
            quote! {
                let #name
            }
        }
    }
}

impl Property {
    pub fn connect_widgets_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        if let PropertyType::Widget(widget) = &self.ty {
            let component_tokens = widget.widget_assignment();
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

            if let Some(returned_widget) = &widget.returned_widget {
                let return_stream = returned_widget.return_assign_tokens();
                stream.extend(quote! {
                    #return_stream = #inner_stream
                });
            } else {
                stream.extend(inner_stream);
            }
        }
    }
}
