use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{Path, Type, Visibility};

use crate::widgets::{PropertyType, ReturnedWidget, TopLevelWidget, Widget};

#[derive(Default)]
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
    pub update_view: TokenStream2,
    /// The tokens for connecting events.
    pub connect: TokenStream2,
}

impl TopLevelWidget {
    pub(super) fn generate_streams(
        &self,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) -> TokenStreams {
        let mut streams = TokenStreams::default();
        self.inner
            .init_token_generation(&mut streams, vis, model_type, relm4_path);

        streams
    }
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
        self.init_stream(&mut streams.init_root);
        name.to_tokens(&mut streams.init_root);

        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.return_stream(&mut streams.return_fields);

        // Rename the `root` to the actual widget name
        streams.rename_root.extend(quote_spanned! {
            name_span => let #name = root.clone();
        });

        for prop in &self.properties.properties {
            prop.init_stream(&mut streams.init_widgets);
            prop.assign_stream(&mut streams.assign_properties, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.return_stream(&mut streams.return_fields);
            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
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
        for prop in &self.properties.properties {
            prop.init_stream(&mut streams.init_widgets);
            prop.assign_stream(&mut streams.assign_properties, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.return_stream(&mut streams.return_fields);
            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
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
            prop.init_stream(&mut streams.init_widgets);
            prop.assign_stream(&mut streams.assign_properties, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.return_stream(&mut streams.return_fields);
            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);
            }
        }
    }
}
