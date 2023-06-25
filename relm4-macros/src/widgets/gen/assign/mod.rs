use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;

use crate::widgets::{Property, PropertyType};

mod assign_property;
mod conditional_widget;
mod properties;
mod signal_handler;
mod widgets;

pub(crate) struct AssignInfo<'a> {
    pub(crate) stream: &'a mut TokenStream2,
    pub(crate) widget_name: &'a Ident,
    pub(crate) template_name: Option<&'a Ident>,
    pub(crate) is_conditional: bool,
}

impl Property {
    fn assign_stream<'a>(&'a self, info: &mut AssignInfo<'a>, sender_name: &'a Ident) {
        match &self.ty {
            PropertyType::Assign(assign) => {
                assign.conditional_assign_stream(info, &self.name, true);
            }
            PropertyType::Widget(widget) => {
                widget.assign_stream(info, &self.name, sender_name);
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.assign_stream(info, &self.name, sender_name);
            }
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.connect_signals_stream(info, &self.name, sender_name);
            }
            PropertyType::ParseError(_) => (),
        }
    }
}
