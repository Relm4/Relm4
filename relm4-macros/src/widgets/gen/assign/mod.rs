use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, Path};

use crate::widgets::{Property, PropertyType};

mod assign_property;
mod conditional_widget;
mod properties;
mod widgets;

impl Property {
    fn assign_stream(&self, stream: &mut TokenStream2, w_name: &Ident, relm4_path: &Path) {
        match &self.ty {
            PropertyType::Assign(assign) => {
                assign.assign_stream(stream, &self.name, w_name, relm4_path)
            }
            PropertyType::Widget(widget) => {
                widget.assign_stream(stream, &self.name, w_name, relm4_path)
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.assign_stream(stream, &self.name, w_name, relm4_path)
            }
            PropertyType::ParseError(_) | PropertyType::SignalHandler(_) => (),
        }
    }
}
