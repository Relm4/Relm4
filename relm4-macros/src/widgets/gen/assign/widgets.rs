use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, Path};

use crate::widgets::{PropertyName, ReturnedWidget, Widget};

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

impl Widget {
    pub(super) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        let assign_fn = p_name.assign_fn_stream(w_name, relm4_path);
        let self_assign_args = p_name.assign_args_stream(w_name);
        let assign = self.widget_assignment();
        let args = self.args.as_ref();
        let span = p_name.span();

        stream.extend(if let Some(ret_widget) = &self.returned_widget {
            let return_assign_stream = ret_widget.return_assign_tokens();
            let unwrap = ret_widget.is_optional.then(|| quote! { .unwrap() });
            quote_spanned! {
                span => #return_assign_stream #assign_fn(#self_assign_args #assign #args) #unwrap;
            }
        } else {
            quote_spanned! {
                span => #assign_fn(#self_assign_args #assign #args);
            }
        });
    }
}
