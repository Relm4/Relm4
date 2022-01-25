use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

use super::{Property, PropertyName, PropertyType, ReturnedWidget, Tracker, Widget, WidgetFunc};

/// Utility methods and functions.
mod util;

/// Generate struct fields.
mod struct_fields;

/// Initialize widgets.
mod init_widgets;

/// Intialize property values.
mod init_properties;

/// Connect events.
mod connect;

/// Fields of the returned widget sturct.
mod return_fields;

/// View stream (mainly for watch!).
mod view;

/// Additional view stream for track!.
mod track;

/// Connect the widgets.
mod connect_widgets;

/// Connect components and widgets.
mod connect_components;

/// Connect to parent properties.
mod parent;

impl Widget {
    pub fn widget_assignment(&self) -> TokenStream2 {
        let w_name = &self.name;

        let ref_token = &self.ref_token;
        let deref_token = &self.deref_token;

        let out_stream = quote! { #ref_token #deref_token #w_name };

        if let Some(wrapper) = &self.wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }
}
