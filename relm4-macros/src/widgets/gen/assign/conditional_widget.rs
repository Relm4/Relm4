use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Ident, Path};

use crate::widgets::{ConditionalBranches, ConditionalWidget, PropertyName};

impl ConditionalWidget {
    pub(super) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        let assign_fn = p_name.assign_fn_stream(w_name, relm4_path);
        let self_assign_args = p_name.assign_args_stream(w_name);
        let span = p_name.span();

        let args = self.args.as_ref().map(|args| {
            quote! {
               , #args
            }
        });

        let w_name = &self.name;
        let assign_args = if let Some(assign_wrapper) = &self.assign_wrapper {
            quote! { #assign_wrapper (&#w_name ) }
        } else {
            quote_spanned! { w_name.span() => &#w_name }
        };

        stream.extend(quote_spanned! {
            span => #assign_fn(#self_assign_args #assign_args #args);
        });

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    let p_name = PropertyName::Ident(Ident::new("add_named", p_name.span()));
                    branch
                        .widget
                        .assign_stream(stream, &p_name, w_name, true, relm4_path);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    let p_name = PropertyName::Ident(Ident::new("add_named", p_name.span()));
                    arm.widget
                        .assign_stream(stream, &p_name, w_name, true, relm4_path);
                }
            }
        }
    }
}
