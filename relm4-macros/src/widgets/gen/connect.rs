use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident};

use super::{Property, PropertyType};

impl PropertyType {
    fn connect_assign_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Connect(closure) = self {
            Some(closure.to_token_stream())
        } else {
            None
        }
    }
}

impl Property {
    pub fn connect_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        if let Some(p_assign) = self.ty.connect_assign_tokens() {
            let p_name = &self.name;
            let p_span = p_name.span().unwrap().into();

            let assign_fn = self.name.assign_fn_stream(&self.generics, w_name);
            let self_assign_args = self.name.assign_args_stream(w_name);

            let mut clone_stream = TokenStream2::new();
            if let Some(args) = &self.args {
                for arg in &args.inner {
                    clone_stream.extend(quote_spanned! { arg.span() =>
                        #[allow(clippy::redundant_clone)]
                        #[allow(clippy::clone_on_copy)]
                        let #arg = #arg.clone();
                    });
                }
            }

            stream.extend(quote_spanned! {
                p_span => {
                    #clone_stream
                    #assign_fn(#self_assign_args #p_assign);
                }
            });
        }
    }
}
