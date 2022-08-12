use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

use crate::widgets::{Property, PropertyType};

mod assign_property;
mod conditional_widget;
mod properties;
mod widgets;

impl Property {
    fn assign_stream(&self, stream: &mut TokenStream2, w_name: &Ident, is_conditional: bool) {
        match &self.ty {
            PropertyType::Assign(assign) => {
                assign.conditional_assign_stream(stream, &self.name, w_name, is_conditional);
            }
            PropertyType::Widget(widget) => {
                widget.assign_stream(stream, &self.name, w_name, is_conditional);
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.assign_stream(stream, &self.name, w_name);
            }
            PropertyType::ParseError(_) | PropertyType::SignalHandler(_) => (),
        }
    }
}
