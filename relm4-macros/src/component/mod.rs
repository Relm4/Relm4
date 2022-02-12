use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Error, PathArguments, Type};
use syn::{Path, Visibility};

use crate::{macros::Macros, util::self_type, ItemImpl};

mod funcs;
mod inject_view_code;
pub(crate) mod token_streams;
mod types;

use inject_view_code::inject_view_code;

pub(crate) fn generate_tokens(
    visibility: Option<Visibility>,
    relm4_path: Path,
    data: ItemImpl,
) -> TokenStream2 {
    if PathArguments::None != data.trait_.segments.last().unwrap().arguments {
        return Error::new(
            data.trait_.segments.span(),
            "Expected no generic parameters for model and parent model",
        )
        .to_compile_error();
    };

    // Create a `Self` type for the model
    let model_type: Type = self_type();

    let types::Types {
        widgets: widgets_type,
        init_params,
        input,
        output,
    } = match types::Types::new(data.types) {
        Ok(types) => types,
        Err(err) => return err.to_compile_error(),
    };

    let trait_ = data.trait_;
    let ty = data.self_ty;
    let outer_attrs = &data.outer_attrs;

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
        init_parts,
        pre_view,
        post_view,
        unhandled_fns,
    } = match funcs::Funcs::new(data.funcs) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    let _root_widget_name = &widgets.name;
    let root_widget_type = widgets.func.type_token_stream();

    let mut streams = token_streams::TokenStreams::default();
    widgets.init_token_generation(&mut streams, &visibility, &model_type, &relm4_path);

    let token_streams::TokenStreams {
        init_root,
        rename_root,
        struct_fields,
        init_widgets,
        assign_properties,
        connect,
        return_fields,
        //parent,
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

    let view_code = quote! {
        #rename_root
        #menus_stream
        #init_widgets
        #assign_properties
        #connect
        #connect_components
    };

    let widgets_return_code = quote! {
        Self::Widgets {
            #return_fields
            #additional_fields_return_stream
        }
    };

    let init_parts_injected = inject_view_code(init_parts, view_code, widgets_return_code);

    quote! {
        #[allow(dead_code)]
        #outer_attrs
        #visibility struct #widgets_type {
            #struct_fields
            #additional_fields
        }

        impl #impl_generics #trait_ for #ty #where_clause {
            type Root = #root_widget_type;
            type Widgets = #widgets_type;

            #init_params
            #input
            #output

            fn init_root() -> Self::Root {
                #init_root
            }

            #init_parts_injected

            /// Update the view to represent the updated model.
            fn update_view(
                &self,
                widgets: &mut Self::Widgets,
                input: &mut Sender<Self::Input>,
                output: &mut Sender<Self::Output>,
            ) {
                let model = self;
                // Wrap pre_view and post_view code to prevent early returns from skipping other view code.
                (|| { #pre_view })();
                #view
                #track
                (|| { #post_view })();
            }

            #(#unhandled_fns)*
        }
    }
}
