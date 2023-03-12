use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Ident;

use crate::widgets::{ConditionalBranches, ConditionalWidget, PropertyName};

use super::AssignInfo;

impl ConditionalWidget {
    pub(super) fn assign_stream<'a>(
        &'a self,
        info: &mut AssignInfo<'a>,
        p_name: &PropertyName,
        sender_name: &'a Ident,
    ) {
        let assign_fn = p_name.assign_fn_stream(info.widget_name);
        let self_assign_args = p_name.assign_args_stream(info.widget_name);
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

        info.stream.extend(quote_spanned! {
            span => #assign_fn(#self_assign_args #assign_args #args);
        });

        info.is_conditional = true;
        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    let p_name = PropertyName::Ident(Ident::new("add_named", p_name.span()));
                    branch.widget.assign_stream(info, &p_name, sender_name);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    let p_name = PropertyName::Ident(Ident::new("add_named", p_name.span()));
                    arm.widget.assign_stream(info, &p_name, sender_name);
                }
            }
        }
    }
}
