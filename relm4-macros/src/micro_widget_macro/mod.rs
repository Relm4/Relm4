use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Error, GenericArgument, PathArguments};
use syn::{Path, Visibility};

use crate::{macros::Macros, ItemImpl};

mod funcs;
mod model_types;
mod token_streams;

pub(crate) fn generate_tokens(
    visibility: Option<Visibility>,
    relm4_path: Path,
    data: ItemImpl,
) -> TokenStream2 {
    if !data.types.is_empty() {
        return Error::new(data.types[0].span(), "Didn't expect a type parameter")
            .to_compile_error();
    }

    let trait_generics = if let PathArguments::AngleBracketed(generics) =
        &data.trait_.segments.last().unwrap().arguments
    {
        generics
    } else {
        return Error::new(
            data.trait_.segments.span(),
            "Expected generic parameters for the micro model",
        )
        .to_compile_error();
    };

    let model_types::ModelTypes { model } = match model_types::ModelTypes::new(trait_generics) {
        Ok(model) => model,
        Err(err) => return err.to_compile_error(),
    };

    let trait_ = data.trait_;
    let ty = data.self_ty;
    let outer_attrs = &data.outer_attrs;

    // Find the type of the model

    // This can be unwrapped savely because the path must have at least one segement after parsing successful.
    let path_args = trait_
        .segments
        .last()
        .map(|segment| &segment.arguments)
        .unwrap();

    let model_ty_opt = if let PathArguments::AngleBracketed(angle_args) = path_args {
        if let Some(GenericArgument::Type(model_ty)) = angle_args.args.first() {
            Some(model_ty)
        } else {
            None
        }
    } else {
        None
    };

    let model_type = if let Some(model_type) = model_ty_opt {
        model_type
    } else {
        return Error::new(
            path_args.span(),
            "Expected generic parameters for the micro model",
        )
        .to_compile_error();
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
    } = match funcs::Funcs::new(&data.funcs) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    let root_widget_name = &widgets.name;
    let root_widget_type = widgets.func.type_token_stream();

    let mut streams = token_streams::TokenStreams::default();
    widgets.generate_micro_widget_tokens_recursively(
        &mut streams,
        &visibility,
        model_type,
        &relm4_path,
    );

    let token_streams::TokenStreams {
        struct_fields,
        init_widgets,
        connect_widgets,
        init_properties,
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
        #outer_attrs
        #visibility struct #ty {
            #struct_fields
            #additional_fields
        }

        impl #impl_generics #trait_ for #ty #where_clause {
            type Root = #root_widget_type;

            /// Initialize the UI.
            fn init_view(model: &#model, sender: #relm4_path::Sender<<#model as #relm4_path::MicroModel>::Msg>) -> Self {
                #pre_init
                #init_widgets
                #connect_widgets
                #menus_stream
                #init_properties
                #connect
                #connect_components
                #post_init
                Self {
                    #return_fields
                    #additional_fields_return_stream
                }
            }

            /// Return the root widget.
            fn root_widget(&self) -> Self::Root {
                self.#root_widget_name.clone()
            }

            /// Update the view to represent the updated model.
            fn view(&mut self, model: &#model, sender: #relm4_path::Sender<<#model as #relm4_path::MicroModel>::Msg>) {
                // Wrap pre_view and post_view code to prevent early returns from skipping other view code.
                (|| { #pre_view })();
                #view
                #track
                (|| { #post_view })();
            }
        }
    }
}
