use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{Error, Path, Visibility};

use crate::widgets::{TopLevelWidget, ViewWidgets, Widget};

#[derive(Default)]
pub(crate) struct TokenStreams {
    /// Parsing errors
    pub error: TokenStream2,
    /// Initialize the root widget.
    pub init_root: TokenStream2,
    /// Rename root to the actual widget name.
    pub rename_root: TokenStream2,
    /// The tokens for the struct fields -> name: Type,
    pub struct_fields: TokenStream2,
    /// The tokens initializing the widgets.
    pub init: TokenStream2,
    /// The tokens initializing the properties.
    pub assign: TokenStream2,
    /// The tokens for connecting events.
    pub connect: TokenStream2,
    /// The tokens for the returned struct fields -> name,
    pub return_fields: TokenStream2,
    /// For destructuring the widget struct field
    pub destructure_fields: TokenStream2,
    /// The view tokens (watch! macro)
    pub update_view: TokenStream2,
}

impl ViewWidgets {
    pub fn generate_streams(
        &self,
        vis: &Option<Visibility>,
        relm4_path: &Path,
        standalone_view: bool,
    ) -> TokenStreams {
        let mut streams = TokenStreams::default();

        for top_level_widget in &self.top_level_widgets {
            top_level_widget.generate_streams(&mut streams, vis, relm4_path, standalone_view);
        }

        streams
    }

    /// Generate root type for `Root` parameter in `Component` impl
    pub fn root_type(&self) -> TokenStream2 {
        for top_level_widget in &self.top_level_widgets {
            if top_level_widget.root_attr.is_some() {
                return top_level_widget.inner.func_type_token_stream();
            }
        }
        Error::new(
            self.span,
            "You need to specify the root widget using the `#[root]` attribute.",
        )
        .to_compile_error()
    }
}

impl TopLevelWidget {
    fn generate_streams(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        relm4_path: &Path,
        standalone_view: bool,
    ) {
        self.inner.init_token_generation(
            streams,
            vis,
            relm4_path,
            !standalone_view && self.root_attr.is_some(),
        );
    }
}

impl Widget {
    pub(super) fn init_token_generation(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        relm4_path: &Path,
        generate_init_root_stream: bool,
    ) {
        let name = &self.name;
        let name_span = name.span();

        // Initialize the root
        if generate_init_root_stream {
            // For the `component` macro
            self.init_root_init_streams(&mut streams.init_root, &mut streams.init, relm4_path);
        } else {
            // For the `view!` macro
            self.init_stream(&mut streams.init, relm4_path);
        }

        self.error_stream(&mut streams.error);
        self.start_assign_stream(&mut streams.assign, relm4_path);
        self.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);
        self.return_stream(&mut streams.return_fields);
        self.destructure_stream(&mut streams.destructure_fields);
        self.update_view_stream(&mut streams.update_view, relm4_path);
        self.connect_signals_stream(&mut streams.connect, relm4_path);

        // Rename the `root` to the actual widget name
        streams.rename_root.extend(quote_spanned! {
            name_span => let #name = root.clone();
        });
    }
}
