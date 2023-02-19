use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse_quote;
use syn::visit_mut::VisitMut;

use crate::attrs::Attrs;
use crate::token_streams::{TokenStreams, TraitImplDetails};
use crate::util;
use crate::visitors::{ComponentVisitor, PreAndPostView, ViewOutputExpander};

pub(crate) fn generate_tokens(
    global_attributes: Attrs,
    mut component_impl: syn::ItemImpl,
) -> TokenStream2 {
    let Attrs {
        visibility,
        asyncness,
    } = global_attributes;

    let mut errors = vec![];

    let mut component_visitor = ComponentVisitor::new(&mut errors);

    component_visitor.visit_item_impl_mut(&mut component_impl);

    let additional_fields = component_visitor.additional_fields.take();

    let menus_stream = component_visitor
        .menus
        .take()
        .map(|menus| menus.menus_stream());

    let mut struct_fields = None;

    match &component_visitor.view_widgets {
        None => component_visitor.errors.push(syn::Error::new_spanned(
            &component_impl,
            "expected `view!` macro invocation",
        )),
        Some(Err(e)) => component_visitor.errors.push(e.clone()),
        _ => (),
    }

    if let ComponentVisitor {
        view_widgets: Some(Ok(view_widgets)),
        model_name: Some(model_name),
        root_name: Some(root_name),
        sender_name: Some(sender_name),
        errors,
        ..
    } = component_visitor
    {
        let trait_impl_details = TraitImplDetails {
            vis: visibility.clone(),
            model_name,
            sender_name,
            root_name: Some(root_name),
        };

        let TokenStreams {
            error,
            init_root,
            rename_root,
            struct_fields: struct_fields_stream,
            init: init_widgets,
            assign,
            connect,
            return_fields,
            destructure_fields,
            update_view,
        } = view_widgets.generate_streams(&trait_impl_details, false);

        let model_name = trait_impl_details.model_name;

        struct_fields = Some(struct_fields_stream);
        let root_widget_type = view_widgets.root_type();

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
            #connect
            {
                #error
            }
            #assign
        };

        let widgets_return_code = parse_quote! {
            Self::Widgets {
                #return_fields
                #additional_fields_return_stream
            }
        };

        ViewOutputExpander::expand(&mut component_impl, view_code, widgets_return_code, errors);

        component_impl.items.push(parse_quote! {
            type Root = #root_widget_type;
        });

        let ty: syn::Type = parse_quote!(Self::Root);
        let init_root = util::verbatim_impl_item_method("init_root", Vec::new(), ty, init_root);
        component_impl.items.push(init_root);

        let PreAndPostView {
            pre_view,
            post_view,
            ..
        } = PreAndPostView::extract(&mut component_impl, errors);

        let sender_ty: syn::TypePath = if asyncness.is_some() {
            parse_quote! { relm4::AsyncComponentSender }
        } else {
            parse_quote! { relm4::ComponentSender }
        };

        component_impl.items.push(parse_quote! {
            /// Update the view to represent the updated model.
            fn update_view(
                &self,
                widgets: &mut Self::Widgets,
                sender: #sender_ty<Self>,
            ) {
                struct __DoNotReturnManually;

                let _no_manual_return: __DoNotReturnManually = (move || {
                    #[allow(unused_variables)]
                    let Self::Widgets {
                        #destructure_fields
                        #additional_fields_return_stream
                    } = widgets;

                    #[allow(unused_variables)]
                    let #model_name = self;

                    #(#pre_view)*
                    #update_view
                    // In post_view returning early is ok
                    (move || { #(#post_view)* })();

                    __DoNotReturnManually
                })();
            }
        });
    }

    // Use the widget type if used.
    let widgets_name = util::generate_widgets_type(
        component_visitor.widgets_ty,
        &mut component_impl,
        &mut errors,
    );

    let widgets_struct = widgets_name.map(|widgets_name| {
        let outer_attrs = &component_impl.attrs;
        quote! {
            #[allow(dead_code)]
            #(#outer_attrs)*
            #[derive(Debug)]
            #visibility struct #widgets_name {
                #struct_fields
                #additional_fields
            }
        }
    });

    let errors = errors.iter().map(syn::Error::to_compile_error);

    let async_trait = asyncness.map(
        |async_token| quote_spanned!(async_token.span => #[relm4::async_trait::async_trait(?Send)]),
    );

    quote! {
        #widgets_struct

        #async_trait
        #component_impl

        #(#errors)*
    }
}
