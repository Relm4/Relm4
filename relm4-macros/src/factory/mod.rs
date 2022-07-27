use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Ident, Visibility};

use crate::component::token_streams;
use crate::macros::Macros;

mod funcs;
mod inject_view_code;
mod types;

use inject_view_code::inject_view_code;

pub(crate) fn generate_tokens(
    vis: Option<Visibility>,
    factory_impl: syn::ItemImpl,
) -> TokenStream2 {
    let types = factory_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Type(ty) => Some(ty.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let types::Types {
        widgets: widgets_type,
        other_types,
    } = match types::Types::new(types) {
        Ok(types) => types,
        Err(err) => return err.to_compile_error(),
    };

    let trait_ = match &factory_impl.trait_ {
        Some((None, path, _)) => path,
        _ => {
            return syn::Error::new_spanned(&factory_impl, "must be a positive trait impl")
                .into_compile_error()
        }
    };
    let ty = &factory_impl.self_ty;
    let outer_attrs = &factory_impl.attrs;

    let macros = factory_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Macro(mac) => Some(mac.mac.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Macros {
        view_widgets,
        additional_fields,
        menus,
    } = match Macros::new(&macros, factory_impl.span().unwrap()) {
        Ok(macros) => macros,
        Err(err) => return err.to_compile_error(),
    };

    // Generate menu tokens
    let menus_stream = menus.map(|m| m.menus_stream());

    let funcs = factory_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Method(func) => Some(func.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let funcs::Funcs {
        init_widgets,
        pre_view,
        post_view,
        unhandled_fns,
        root_name,
    } = match funcs::Funcs::new(funcs) {
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
        &Ident::new("self", Span2::call_site()),
        Some(&root_name),
        false,
    );

    let root_widget_type = view_widgets.root_type();

    let impl_generics = &factory_impl.generics.params;
    let where_clause = &factory_impl.generics.where_clause;

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

    let last_segment = trait_.segments.last().unwrap();

    let (container_widget, parent_msg) = {
        let mut args = match &last_segment.arguments {
            syn::PathArguments::AngleBracketed(args) => Some(args.args.clone()),
            _ => None,
        }
        .unwrap_or_default()
        .into_iter();
        (
            args.next().unwrap_or_else(|| syn::parse_quote!(())),
            args.next().unwrap_or_else(|| syn::parse_quote!(())),
        )
    };

    let init_injected = inject_view_code(
        init_widgets,
        view_code,
        widgets_return_code,
        &container_widget,
        &parent_msg,
    );

    quote! {
        #[allow(dead_code)]
        #(#outer_attrs)*
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
                sender: &relm4::factory::FactoryComponentSender<#container_widget, #parent_msg, Self>,
            ) {
                #[allow(unused_variables)]
                let Self::Widgets {
                    #destructure_fields
                    #additional_fields_return_stream
                } = widgets;

                // Wrap post_view code to prevent early returns from skipping other view code.
                #pre_view
                #update_view
                (|| { #post_view })();
            }

            #(#unhandled_fns)*
        }
    }
}
