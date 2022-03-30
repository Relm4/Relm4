use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

use crate::widgets::{Property, PropertyType, Widget, WidgetAttr};

impl Property {
    pub fn init_stream(&self, stream: &mut TokenStream2) {
        if let PropertyType::Widget(widget) = &self.ty {
            widget.init_stream(stream);
        }
    }
}

impl Widget {
    pub fn init_stream(&self, stream: &mut TokenStream2) {
        let mutability = &self.mutable;
        let name = &self.name;
        let func = self.func.func_token_stream();
        let span = self.name.span();

        if self.attr == WidgetAttr::None {
            stream.extend(if let Some(ty) = &self.func.ty {
                quote_spanned! {
                    span => let #mutability #name: #ty = #func;
                }
            } else {
                quote_spanned! {
                    span => let #mutability #name = #func;
                }
            });
        }
    }
}
