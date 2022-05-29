use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Ident, Path, PathArguments, Visibility};

use crate::component::token_streams;
use crate::macros::Macros;
use crate::ItemImpl;

mod funcs;
mod inject_view_code;
mod types;

use inject_view_code::inject_view_code;

pub(crate) fn generate_tokens(
    vis: Option<Visibility>,
    relm4_path: Path,
    data: ItemImpl,
) -> TokenStream2 {
    let last_segment = data
        .trait_
        .segments
        .last()
        .expect("Expected at least one segment in the trait path");
    let container_widget = if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
        if let Some(arg) = args.args.first() {
            arg.to_token_stream()
        } else {
            quote! { () }
        }
    } else {
        quote! { () }
    };

    let types::Types {
        widgets: widgets_type,
        other_types,
    } = match types::Types::new(data.types) {
        Ok(types) => types,
        Err(err) => return err.to_compile_error(),
    };

    let trait_ = data.trait_;
    let ty = data.self_ty;
    let outer_attrs = &data.outer_attrs;

    let Macros {
        view_widgets,
        additional_fields,
        menus,
    } = match Macros::new(&data.macros, data.brace_span.unwrap()) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    // Generate menu tokens
    let menus_stream = menus.map(|m| m.menus_stream(&relm4_path));

    let funcs::Funcs {
        init_widgets,
        pre_view,
        post_view,
        unhandled_fns,
        root_name,
    } = match funcs::Funcs::new(data.funcs) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    let token_streams::TokenStreams {
        error,
        init_root,
        rename_root,
        struct_fields,
        init,
        assign,
        connect,
        return_fields,
        destructure_fields,
        update_view,
    } = view_widgets.generate_streams(
        &vis,
        &relm4_path,
        &Ident::new("self", Span2::call_site()),
        Some(&root_name),
        false,
    );

    let root_widget_type = view_widgets.root_type();

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
        #init
        #connect
        {
            #error
        }
        #assign
    };

    let widgets_return_code = quote! {
        Self::Widgets {
            #return_fields
            #additional_fields_return_stream
        }
    };

    let init_injected = inject_view_code(
        init_widgets,
        view_code,
        widgets_return_code,
        container_widget,
        &relm4_path,
    );

    quote! {
        #[allow(dead_code)]
        #outer_attrs
        #[derive(Debug)]
        #vis struct #widgets_type {
            #struct_fields
            #additional_fields
        }

        impl #impl_generics #trait_ for #ty #where_clause {
            type Root = #root_widget_type;
            type Widgets = #widgets_type;

            #(#other_types)*

            fn init_root(&self) -> Self::Root {
                #init_root
            }

            #init_injected

            /// Update the view to represent the updated model.
            fn update_view(
                &self,
                widgets: &mut Self::Widgets,
                input: &Sender<Self::Input>,
                output: &Sender<Self::Output>
            ) {
                #[allow(unused_variables)]
                let Self::Widgets {
                    #destructure_fields
                    #additional_fields_return_stream
                } = widgets;

                // Wrap pre_view and post_view code to prevent early returns from skipping other view code.
                #[warn(unreachable_code)]
                #pre_view
                #update_view
                (|| { #post_view })();
            }

            #(#unhandled_fns)*
        }
    }
}
