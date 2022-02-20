use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{Path, Type, Visibility};

use crate::widgets::{PropertyType, ReturnedWidget, Widget};

#[derive(Debug, Default)]
pub(crate) struct TokenStreams {
    /// Initialize the root widget.
    pub init_root: TokenStream2,
    /// Rename root to the actual widget name.
    pub rename_root: TokenStream2,
    /// The tokens for the struct fields -> name: Type,
    pub struct_fields: TokenStream2,
    /// The tokens initializing the widgets.
    pub init_widgets: TokenStream2,
    /// The tokens initializing the properties.
    pub assign_properties: TokenStream2,
    /// The tokens for the returned struct fields -> name,
    pub return_fields: TokenStream2,
    /// The view tokens (watch! macro)
    pub view: TokenStream2,
    /// The view tokens (track! macro)
    pub track: TokenStream2,
    /// The tokens for connecting events.
    pub connect: TokenStream2,
    /// The tokens for connecting events to components.
    pub connect_components: TokenStream2,
}

impl Widget {
    pub(super) fn init_token_generation(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) {
        let name = &self.name;
        let name_span = name.span();

        // Initialize the root
        self.init_widgets_stream(&mut streams.init_root);
        name.to_tokens(&mut streams.init_root);

        // Rename the `root` to the actual widget name
        streams.rename_root.extend(quote_spanned! {
            name_span => let #name = root.clone();
        });

        // The root isn't part of the widgets struct
        // self.struct_fields_stream(&mut streams.struct_fields, vis);
        // self.return_stream(&mut streams.return_fields);

        for prop in &self.properties.properties {
            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
            } else {
                prop.property_init_stream(&mut streams.assign_properties, &self.name, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);

                prop.view_stream(&mut streams.view, &self.name, relm4_path, true);
                prop.track_stream(&mut streams.track, &self.name, model_type, true, relm4_path);

                prop.connect_stream(&mut streams.connect, &self.name, relm4_path);
                prop.connect_component_stream(
                    &mut streams.connect_components,
                    &self.name,
                    relm4_path,
                );

                //prop.connect_parent_stream(&mut streams.parent, &self.name);
            }
        }
    }

    fn generate_component_tokens_recursively(
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
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
            } else {
                prop.property_init_stream(&mut streams.assign_properties, &self.name, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);

                prop.view_stream(&mut streams.view, &self.name, relm4_path, true);
                prop.track_stream(&mut streams.track, &self.name, model_type, true, relm4_path);

                prop.connect_stream(&mut streams.connect, &self.name, relm4_path);
                prop.connect_component_stream(
                    &mut streams.connect_components,
                    &self.name,
                    relm4_path,
                );

                //prop.connect_parent_stream(&mut streams.parent, &self.name);
            }
        }
    }
}

impl ReturnedWidget {
    fn generate_component_tokens_recursively(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) {
        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.return_stream(&mut streams.return_fields);

        for prop in &self.properties.properties {
            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);
            } else {
                prop.property_init_stream(&mut streams.assign_properties, &self.name, relm4_path);
                prop.connect_widgets_stream(&mut streams.assign_properties, &self.name, relm4_path);

                prop.connect_stream(&mut streams.connect, &self.name, relm4_path);

                prop.view_stream(&mut streams.view, &self.name, relm4_path, false);
                prop.track_stream(
                    &mut streams.track,
                    &self.name,
                    model_type,
                    false,
                    relm4_path,
                );

                prop.connect_component_stream(
                    &mut streams.connect_components,
                    &self.name,
                    relm4_path,
                );
            }
        }
    }
}
