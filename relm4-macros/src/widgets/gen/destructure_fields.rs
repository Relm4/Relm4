use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::{Property, PropertyType, ReturnedWidget, SignalHandler, Widget};

impl Property {
    pub fn destructure_stream(&self, stream: &mut TokenStream2) {
        match &self.ty {
            PropertyType::Assign(_) => (),
            PropertyType::Widget(widget) => widget.destructure_stream(stream),
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.destructure_stream(stream)
            }
        }
    }
}

impl Widget {
    pub fn destructure_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;

        stream.extend(quote! { #name, });
    }
}

impl ReturnedWidget {
    pub fn destructure_stream(&self, stream: &mut TokenStream2) {
        if self.ty.is_some() {
            let name = &self.name;
            stream.extend(quote! {
                #name,
            });
        }
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
