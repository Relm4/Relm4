use crate::widgets::{ConditionalBranches, ConditionalWidget, PropertyName};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::__private::TokenStream2;
use syn::Ident;
use syn::spanned::Spanned;

use super::AssignInfo;

impl ConditionalWidget {
    pub(crate) fn start_assign_stream<'a>(
        &'a self,
        stream: &'a mut TokenStream2,
        sender_name: &'a Ident,
    ) {
        let w_name = &self.name;
        let mut info = AssignInfo {
            stream,
            widget_name: w_name,
            template_path: None,
            is_conditional: true,
        };
        let span = info.stream.span();

        self.assign_branches(&mut info, sender_name, span);

        if let Some(properties) = &self.properties {
            properties.assign_stream(&mut info, sender_name);
        }
    }

    pub(crate) fn assign_stream<'a>(
        &'a self,
        info: &mut AssignInfo<'a>,
        p_name: &PropertyName,
        sender_name: &'a Ident,
    ) {
        let assign_fn = p_name.assign_fn_stream(info);
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

        let mut info = AssignInfo {
            stream: info.stream,
            widget_name: &self.name,
            template_path: None,
            is_conditional: true,
        };

        self.assign_branches(&mut info, sender_name, span);

        if let Some(properties) = &self.properties {
            properties.assign_stream(&mut info, sender_name);
        }
    }

    pub(crate) fn assign_branches<'a>(
        &'a self,
        info: &mut AssignInfo<'a>,
        sender_name: &'a Ident,
        span: Span,
    ) {
        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    let p_name = PropertyName::Ident(Ident::new("add_named", span));
                    branch.widget.assign_stream(info, &p_name, sender_name);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    let p_name = PropertyName::Ident(Ident::new("add_named", span));
                    arm.widget.assign_stream(info, &p_name, sender_name);
                }
            }
        }
    }
}
