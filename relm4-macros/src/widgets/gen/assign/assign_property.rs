use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{spanned::Spanned, Ident, Path};

use crate::widgets::{AssignProperty, PropertyName};

impl AssignProperty {
    pub fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        let assign_fn = p_name.assign_fn_stream(w_name, relm4_path);
        let self_assign_args = p_name.assign_args_stream(w_name);
        let assign = &self.expr;
        let args = self.args.as_ref();
        let span = p_name.span();

        stream.extend(match (self.optional_assign, self.iterative) {
            (false, false) => {
                quote_spanned! {
                    span => #assign_fn(#self_assign_args #assign #args);
                }
            }
            (true, false) => {
                quote_spanned! {
                    span => if let Some(__p_assign) = #assign {
                        #assign_fn(#self_assign_args __p_assign #args);
                    }
                }
            }
            (false, true) => {
                quote_spanned! {
                    span => for __elem in #assign {
                        #assign_fn(#self_assign_args __elem #args);
                    }
                }
            }
            (true, true) => {
                quote_spanned! {
                    span => for __elem in #assign {
                        if let Some(__p_assign) = __elem {
                            #assign_fn(#self_assign_args __p_assign #args);
                        }
                    }
                }
            }
        });
    }
}
