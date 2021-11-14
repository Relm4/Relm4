use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Path, Type, Visibility};

use super::{
    Property, PropertyName, PropertyType, ReturnedWidget, TokenStreams, Tracker, Widget, WidgetFunc,
};

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

/// Component intitialization.
mod components;

/// Connect components and widgets.
mod connect_components;

impl Widget {
    pub fn generate_tokens_recursively(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) {
        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.init_widgets_stream(&mut streams.init_widgets);
        self.return_stream(&mut streams.return_fields);

        for prop in &self.properties.properties {
            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_tokens_recursively(streams, vis, model_type, relm4_path);
                // if let Some(returned_widget) = widget.returned_widget {
                //     returned_widget.get_widget_list(widgets);
                // }
            } else {
                prop.property_init_stream(&mut streams.init_properties, &self.name, relm4_path);
                prop.connect_stream(&mut streams.connect, &self.name);

                prop.view_stream(&mut streams.view, &self.name, relm4_path);
                prop.track_stream(&mut streams.track, &self.name, model_type);

                prop.connect_component_stream(&mut streams.connect_components, &self.name);
            }
            // Component stream needs to be generated for widgets, too.
            prop.component_stream(&mut streams.components, &self.name);
        }
    }

    pub fn widget_assignment(&self) -> TokenStream2 {
        let w_span = self.func.span();
        let w_name = &self.name;
        let out_stream = if self.assign_as_ref {
            quote_spanned! { w_span => & #w_name }
        } else {
            quote! { #w_name }
        };
        if let Some(wrapper) = &self.wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }
}

// impl ReturnedWidget {
//     pub fn generate_tokens_recursively(
//         &self,
//         streams: &mut TokenStreams,
//         vis: &Option<Visibility>,
//         model_type: &Type,
//         relm4_path: &Path,
//     ) {
//         self.struct_fields_stream(&mut streams.struct_fields, vis);
//         self.init_widgets_stream(&mut streams.init_widgets);
//         self.return_stream(&mut streams.return_fields);

//         for prop in &self.properties.properties {
//             if let PropertyType::Widget(widget) = &prop.ty {
//                 widget.generate_tokens_recursively(streams, vis, model_type, relm4_path);
// if let Some(returned_widget) = widget.returned_widget {
//     returned_widget.get_widget_list(widgets);
// }
//             } else {
//                 prop.property_init_stream(&mut streams.init_properties, &self.name, relm4_path);
//                 prop.connect_stream(&mut streams.connect, &self.name);

//                 prop.view_stream(&mut streams.view, &self.name, relm4_path);
//                 prop.track_stream(&mut streams.track, &self.name, model_type);

//                 prop.connect_component_stream(&mut streams.connect_components, &self.name);
//             }
// Component stream needs to be generated for widgets, too.
//             prop.component_stream(&mut streams.components, &self.name);
//         }
//     }
// }
