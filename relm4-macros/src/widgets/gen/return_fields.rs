use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::widgets::{Property, PropertyType, ReturnedWidget, SignalHandler, Widget, WidgetAttr};

impl Property {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        match &self.ty {
            PropertyType::Assign(_) => (),
            PropertyType::Widget(widget) => widget.return_stream(stream),
            PropertyType::SignalHandler(signal_handler) => signal_handler.return_stream(stream),
        }
    }
}

impl Widget {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;

        stream.extend(if self.attr == WidgetAttr::LocalRef {
            // The local reference must be cloned first
            quote! { #name: #name.clone(), }
        } else {
            quote! { #name, }
        });
    }
}

impl ReturnedWidget {
    pub fn return_stream(&self, stream: &mut TokenStream2) {
        self.destructure_stream(stream);
    }
}

impl SignalHandler {
    fn return_stream(&self, stream: &mut TokenStream2) {
        self.destructure_stream(stream);
    }
}
