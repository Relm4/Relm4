use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Ident;

use crate::widgets::{PropertyName, ReturnedWidget, Widget, WidgetTemplateAttr};

use super::AssignInfo;

impl ReturnedWidget {
    fn return_assign_tokens(&self) -> TokenStream2 {
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
            is_conditional: false,
        };
        self.properties.assign_stream(&mut info, sender_name);
    }

    pub(super) fn assign_stream<'a>(
        &'a self,
        info: &mut AssignInfo<'a>,
        p_name: &PropertyName,
        sender_name: &'a Ident,
    ) {
        // Recursively generate code for properties
        {
            let template_path = (self.template_attr == WidgetTemplateAttr::TemplateChild)
                .then_some(self.func.widget_template_path(info.widget_name, &self.name));

            let mut info = AssignInfo {
                stream: info.stream,
                widget_name: &self.name,
                template_path,
                is_conditional: info.is_conditional,
            };
            self.properties.assign_stream(&mut info, sender_name);
        }

        // Template children are already assigned by the template.
        if self.template_attr != WidgetTemplateAttr::TemplateChild {
            let assign_fn = p_name.assign_fn_stream(info);
            let self_assign_args = p_name.assign_args_stream(info.widget_name);
            let assign = self.widget_assignment();
            let span = p_name.span();

            let args = self.args.as_ref().map(|args| {
                quote_spanned! {
                   args.span() => ,#args
                }
            });

            info.stream.extend(if let Some(ret_widget) = &self.returned_widget {
                let return_assign_stream = ret_widget.return_assign_tokens();
                let unwrap = ret_widget.is_optional.then(|| quote! { .unwrap() });
                quote_spanned! {
                    span => #return_assign_stream = #assign_fn(#self_assign_args #assign #args) #unwrap;
                }
            } else {
                quote_spanned! {
                    span => #assign_fn(#self_assign_args #assign #args);
                }
            });
        }

        if let Some(returned_widget) = &self.returned_widget {
            let mut info = AssignInfo {
                stream: info.stream,
                widget_name: &returned_widget.name,
                template_path: None,
                is_conditional: info.is_conditional,
            };
            returned_widget
                .properties
                .assign_stream(&mut info, sender_name);
        }
    }
}
