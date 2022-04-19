use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Path, Type, Visibility};

use crate::{macros::Macros, util::self_type, ItemImpl};

mod funcs;
mod token_streams;
mod types;

pub(crate) fn generate_tokens(
    visibility: Option<Visibility>,
    relm4_path: Path,
    data: ItemImpl,
) -> TokenStream2 {
    let trait_ = data.trait_;
    let ty = data.self_ty;
    let outer_attrs = &data.outer_attrs;

    // Create a `Self` type for the model
    let model_type: Type = self_type();

    let types::Types {
        factory: factory_ty,
        widget: widget_ty,
        view: view_ty,
        msg: msg_ty,
    } = match types::Types::new(data.types) {
        Ok(types) => types,
        Err(err) => return err.to_compile_error(),
    };

    let Macros {
        widgets,
        additional_fields,
        menus,
    } = match Macros::new(&data.macros, data.brace_span.unwrap()) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    // Generate menu tokens
    let menus_stream = menus.map(|m| m.menus_stream(&relm4_path));

    let funcs::Funcs {
        pre_init,
        post_init,
        pre_view,
        post_view,
        position,
    } = match funcs::Funcs::new(&data.funcs) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    let root_widget_name = &widgets.name;
    let root_widget_type = widgets.func.type_token_stream();

    let mut streams = token_streams::TokenStreams::default();
    widgets.generate_factory_prototype_tokens_recursively(
        &mut streams,
        &visibility,
        &model_type,
        &relm4_path,
    );

    let token_streams::TokenStreams {
        struct_fields,
        init_widgets,
        connect_widgets,
        assign_properties,
        connect,
        return_fields,
        connect_components,
        view,
        track,
    } = streams;

    let impl_generics = data.impl_generics;
    let where_clause = data.where_clause;

    // Extract identifiers from additional fields for struct initialization: "test: u8" => "test"
    let additional_fields_return_stream = if let Some(fields) = &additional_fields {
        let mut tokens = TokenStream2::new();
        for field in fields.inner.pairs() {
            tokens.extend(field.value().ident.to_token_stream());
            tokens.extend(quote! {,});
        }
        tokens
    } else {
        TokenStream2::new()
    };

    quote! {
        #[allow(dead_code)]
        #[derive(Debug)]
        #outer_attrs
        #visibility struct #widget_ty {
            #struct_fields
            #additional_fields
        }

        impl #impl_generics #trait_ for #ty #where_clause {
            #factory_ty
            type Widgets = #widget_ty;
            #view_ty
            #msg_ty
            type Root = #root_widget_type;

            /// Initialize the UI.
            fn init_view(&self, key: &<Self::Factory as #relm4_path::factory::Factory<Self, Self::View>>::Key, sender: #relm4_path::Sender<Self::Msg>) -> Self::Widgets {
                #pre_init
                #init_widgets
                #connect_widgets
                #menus_stream
                #assign_properties
                #connect
                #connect_components
                #post_init

                Self::Widgets {
                    #return_fields
                    #additional_fields_return_stream
                }
            }

            /// Return the root widget.
            fn root_widget(widgets: &Self::Widgets) -> &Self::Root {
                &widgets.#root_widget_name
            }

            /// Update the view to represent the updated model.
            fn view(&self, key: &<Self::Factory as #relm4_path::factory::Factory<Self, Self::View>>::Key, widgets: &Self::Widgets) {
                // Wrap pre_view and post_view code to prevent early returns from skipping other view code.
                (|| { #pre_view })();
                #view
                #track
                (|| { #post_view })();
            }

            #position
        }
    }
}
