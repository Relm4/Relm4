use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned;

use super::Property;

mod property;
mod property_name;
mod property_type;
mod widget_func;

/// Generate tokens for assigning properties.
pub(crate) fn property_assign_tokens(
    stream: &mut TokenStream2,
    prop: &Property,
    assign_fn: TokenStream2,
    self_assign_args: Option<TokenStream2>,
    p_assign: TokenStream2,
    return_assign_stream: Option<TokenStream2>,
    args_stream: Option<TokenStream2>,
) {
    let p_name = &prop.name;
    let p_span = p_name.span().unwrap().into();
    stream.extend(match (prop.optional_assign, prop.iterative) {
        (false, false) => {
            if prop.ty.connect_widget_with_unwrap() {
                quote_spanned! {
                    p_span => #return_assign_stream #assign_fn(#self_assign_args #p_assign #args_stream).unwrap();
                }
            } else {
                quote_spanned! {
                    p_span => #return_assign_stream #assign_fn(#self_assign_args #p_assign #args_stream);
                }
            }
        }
        (true, false) => {
            quote_spanned! {
                p_span => if let Some(__p_assign) = #p_assign {
                    #return_assign_stream #assign_fn(#self_assign_args __p_assign #args_stream);
                }
            }
        }
        (false, true) => {
            assert!(return_assign_stream.is_none(), "Can't use returned value when using iterative assignment");
            quote_spanned! {
                p_span => for __elem in #p_assign {
                    #assign_fn(#self_assign_args __elem #args_stream );
                }
            }
        }
        (true, true) => {
            assert!(return_assign_stream.is_none(), "Can't use returned value when using iterative assignment");
            quote_spanned! {
                p_span => for __elem in #p_assign {
                    if let Some(__p_assign) = __elem {
                        #assign_fn(#self_assign_args __p_assign #args_stream );
                    }
                }
            }
        }
    });
}
