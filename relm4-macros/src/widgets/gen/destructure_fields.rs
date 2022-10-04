use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyType, ReturnedWidget,
    SignalHandler, Widget,
};

impl Property {
    fn destructure_stream(&self, stream: &mut TokenStream2) {
        match &self.ty {
            PropertyType::Widget(widget) => widget.destructure_stream(stream),
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.destructure_stream(stream);
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.destructure_stream(stream);
            }
            PropertyType::Assign(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn destructure_stream(&self, stream: &mut TokenStream2) {
        for prop in &self.properties {
            prop.destructure_stream(stream);
        }
    }
}

impl Widget {
    pub(crate) fn destructure_stream(&self, stream: &mut TokenStream2) {
        if self.has_struct_field() {
            let name = &self.name;

            stream.extend(quote! { #name, });
        }

        self.properties.destructure_stream(stream);
    }
}

impl ConditionalWidget {
    fn destructure_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;

        stream.extend(quote! { #name, });

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.destructure_stream(stream);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.destructure_stream(stream);
                }
            }
        }
    }
}

impl ReturnedWidget {
    pub(super) fn destructure_stream(&self, stream: &mut TokenStream2) {
        if self.ty.is_some() {
            let name = &self.name;
            stream.extend(quote! {
                #name,
            });
        }

        self.properties.destructure_stream(stream);
    }
}

impl SignalHandler {
    pub(super) fn destructure_stream(&self, stream: &mut TokenStream2) {
        if let Some(signal_handler_id) = &self.handler_id {
            stream.extend(quote! {
                #signal_handler_id,
            });
        }
    }
}
