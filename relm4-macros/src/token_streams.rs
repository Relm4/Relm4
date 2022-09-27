use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::{Error, Ident, Visibility};

use crate::widgets::{TopLevelWidget, ViewWidgets, Widget};

#[derive(Default)]
pub(super) struct TokenStreams {
    /// Parsing errors
    pub(super) error: TokenStream2,
    /// Initialize the root widget.
    pub(super) init_root: TokenStream2,
    /// Rename root to the actual widget name.
    pub(super) rename_root: TokenStream2,
    /// The tokens for the struct fields -> name: Type,
    pub(super) struct_fields: TokenStream2,
    /// The tokens initializing the widgets.
    pub(super) init: TokenStream2,
    /// The tokens initializing the properties.
    pub(super) assign: TokenStream2,
    /// The tokens for connecting events.
    pub(super) connect: TokenStream2,
    /// The tokens for the returned struct fields -> name,
    pub(super) return_fields: TokenStream2,
    /// For destructuring the widget struct field
    pub(super) destructure_fields: TokenStream2,
    /// The view tokens (watch! macro)
    pub(super) update_view: TokenStream2,
}

pub(super) struct TraitImplDetails {
    /// The visibility of the widgets struct.
    pub(super) vis: Option<Visibility>,
    /// The name of the model.
    pub(super) model_name: Ident,
    /// The name of the root widget.
    pub(super) root_name: Option<Ident>,
    /// The name of the sender used in the init function.
    pub(super) sender_name: Ident,
}

impl ViewWidgets {
    pub(super) fn generate_streams(
        &self,
        trait_impl_details: &TraitImplDetails,
        standalone_view: bool,
    ) -> TokenStreams {
        let mut streams = TokenStreams::default();

        for top_level_widget in &self.top_level_widgets {
            top_level_widget.generate_streams(&mut streams, trait_impl_details, standalone_view);
        }

        streams
    }

    /// Generate root type for `Root` parameter in `Component` impl
    pub(super) fn root_type(&self) -> TokenStream2 {
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
        trait_impl_details: &TraitImplDetails,
        standalone_view: bool,
    ) {
        self.inner.init_token_generation(
            streams,
            trait_impl_details,
            !standalone_view && self.root_attr.is_some(),
        );
    }
}

impl Widget {
    pub(super) fn init_token_generation(
        &self,
        streams: &mut TokenStreams,
        trait_impl_details: &TraitImplDetails,
        generate_init_root_stream: bool,
    ) {
        let TraitImplDetails {
            vis,
            model_name,
            root_name,
            sender_name,
        } = trait_impl_details;

        let name = &self.name;
        let name_span = name.span();

        // Initialize the root
        if generate_init_root_stream {
            // For the `component` macro
            self.init_root_init_streams(&mut streams.init_root, &mut streams.init);
        } else {
            // For the `view!` macro
            self.init_stream(&mut streams.init);
        }

        self.error_stream(&mut streams.error);
        self.start_assign_stream(&mut streams.assign);
        self.init_conditional_init_stream(&mut streams.assign, model_name);
        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.return_stream(&mut streams.return_fields);
        self.destructure_stream(&mut streams.destructure_fields);
        self.init_update_view_stream(&mut streams.update_view, model_name);
        self.connect_signals_stream(&mut streams.connect, sender_name);

        // Rename the `root` to the actual widget name
        if generate_init_root_stream {
            if let Some(root_name) = root_name {
                streams.rename_root.extend(quote_spanned! {
                    name_span => let #name = #root_name.clone();
                });
            }
        }
    }
}
