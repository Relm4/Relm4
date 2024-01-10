use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::visit_mut::VisitMut;
use syn::{parse_quote, Ident};

use crate::attrs::Attrs;
use crate::token_streams::{TokenStreams, TraitImplDetails};
use crate::util;
use crate::visitors::{FactoryComponentVisitor, PreAndPostView, ViewOutputExpander};

pub(crate) fn generate_tokens(
    global_attributes: Attrs,
    mut factory_impl: syn::ItemImpl,
) -> TokenStream2 {
    let Attrs {
        visibility,
        asyncness,
    } = global_attributes;

    let mut errors = vec![];

    let mut factory_visitor = FactoryComponentVisitor::new(&mut errors);
    factory_visitor.visit_item_impl_mut(&mut factory_impl);

    let additional_fields = factory_visitor.additional_fields.take();

    let menus_stream = factory_visitor.menus.take().map(|m| m.menus_stream());

    let mut struct_fields = None;

    match &factory_visitor.view_widgets {
        None => factory_visitor.errors.push(syn::Error::new_spanned(
            &factory_impl,
            "expected `view!` macro invocation",
        )),
        Some(Err(e)) => factory_visitor.errors.push(e.clone()),
        _ => (),
    }

    // Insert default index type for sync variants
    // if it wasn't specified by the user.
    if factory_visitor.index_ty.is_none() && asyncness.is_none() {
        factory_impl.items.push(parse_quote! {
            type Index = relm4::factory::DynamicIndex;
        });
    }

    if let FactoryComponentVisitor {
        view_widgets: Some(Ok(view_widgets)),
        root_name,
        init_widgets,
        errors,
        ..
    } = factory_visitor
    {
        let TokenStreams {
            error,
            init_root,
            rename_root,
            struct_fields: struct_fields_stream,
            init,
            assign,
            return_fields,
            destructure_fields,
            update_view,
        } = view_widgets.generate_streams(
            &TraitImplDetails {
                vis: visibility.clone(),
                model_name: Ident::new("self", Span2::call_site()),
                root_name: Some(
                    root_name.unwrap_or_else(|| Ident::new("root", Span2::call_site())),
                ),
                sender_name: Ident::new("sender", Span2::call_site()),
            },
            false,
        );

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
            #init
            #assign
            {
                #error
            }
        };

        let widgets_return_code = parse_quote! {
            Self::Widgets {
                #return_fields
                #additional_fields_return_stream
            }
        };

        let sender_ty: Ident = if asyncness.is_some() {
            parse_quote! { AsyncFactorySender }
        } else {
            parse_quote! { FactorySender }
        };

        let index_ty: syn::TypePath = if asyncness.is_some() {
            parse_quote! { relm4::factory::DynamicIndex }
        } else {
            parse_quote! { Self::Index }
        };

        if init_widgets.is_some() {
            ViewOutputExpander::expand(&mut factory_impl, view_code, widgets_return_code, errors);
        } else {
            factory_impl.items.push(parse_quote! {
                fn init_widgets(
                    &mut self,
                    index: & #index_ty,
                    root: Self::Root,
                    returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
                    sender: relm4::factory::#sender_ty<Self>,
                ) -> Self::Widgets {
                    #view_code
                    #widgets_return_code
                }
            });
        }

        factory_impl.items.push(parse_quote! {
            type Root = #root_widget_type;
        });

        let ty: syn::Type = parse_quote!(Self::Root);
        factory_impl.items.push(if asyncness.is_some() {
            util::verbatim_impl_item_fn("init_root", Vec::new(), ty, init_root)
        } else {
            let args = vec![parse_quote! { &self}];
            util::verbatim_impl_item_fn("init_root", args, ty, init_root)
        });

        let PreAndPostView {
            pre_view,
            post_view,
            ..
        } = PreAndPostView::extract(&mut factory_impl, errors);

        factory_impl.items.push(parse_quote! {
            // Update the view to represent the updated model.
            fn update_view(
                &self,
                widgets: &mut Self::Widgets,
                sender: relm4::factory::#sender_ty<Self>,
            ) {
                struct __DoNotReturnManually;

                let _no_manual_return: __DoNotReturnManually = (move || {
                    #[allow(unused_variables)]
                    let Self::Widgets {
                        #destructure_fields
                        #additional_fields_return_stream
                    } = widgets;

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
    let widgets_name =
        util::generate_widgets_type(factory_visitor.widgets_ty, &mut factory_impl, &mut errors);

    let widgets_struct = widgets_name.map(|ty| {
        let outer_attrs = &factory_impl.attrs;
        quote! {
            #[allow(dead_code)]
            #(#outer_attrs)*
            #[derive(Debug)]
            #visibility struct #ty {
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
        #factory_impl

        #(#errors)*
    }
}
