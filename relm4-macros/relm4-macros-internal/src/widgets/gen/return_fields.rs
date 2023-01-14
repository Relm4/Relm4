use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyType, ReturnedWidget,
    SignalHandler, Widget, WidgetAttr,
};

impl Property {
    fn return_stream(&self, stream: &mut TokenStream2) {
        match &self.ty {
            PropertyType::Widget(widget) => widget.return_stream(stream),
            PropertyType::SignalHandler(signal_handler) => signal_handler.return_stream(stream),
            PropertyType::ConditionalWidget(cond_widget) => cond_widget.return_stream(stream),
            PropertyType::Assign(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn return_stream(&self, stream: &mut TokenStream2) {
        for prop in &self.properties {
            prop.return_stream(stream);
        }
    }
}

impl Widget {
    pub(crate) fn return_stream(&self, stream: &mut TokenStream2) {
        if self.has_struct_field() {
            let name = &self.name;

            stream.extend(if self.attr == WidgetAttr::LocalRef {
                // The local reference must be cloned first
                quote! { #name: #name.clone(), }
            } else {
                quote! { #name, }
            });
        }

        self.properties.return_stream(stream);
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.return_stream(stream);
        }
    }
}

impl ConditionalWidget {
    fn return_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;

        stream.extend(quote! { #name, });

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.return_stream(stream);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.return_stream(stream);
                }
            }
        }
    }
}

impl ReturnedWidget {
    fn return_stream(&self, stream: &mut TokenStream2) {
        self.destructure_stream(stream);
        self.properties.return_stream(stream);
    }
}

impl SignalHandler {
    fn return_stream(&self, stream: &mut TokenStream2) {
        self.destructure_stream(stream);
    }
}
