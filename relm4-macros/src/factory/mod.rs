use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::visit_mut::VisitMut;
use syn::{parse_quote, Ident, Visibility};

use crate::token_streams::{TokenStreams, TraitImplDetails};
use crate::visitors::{FactoryComponentVisitor, PreAndPostView, ViewOutputExpander};

pub(crate) fn generate_tokens(
    vis: &Option<Visibility>,
    mut factory_impl: syn::ItemImpl,
) -> TokenStream2 {
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
            connect,
            return_fields,
            destructure_fields,
            update_view,
        } = view_widgets.generate_streams(
            &TraitImplDetails {
                vis: vis.clone(),
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

        if init_widgets.is_some() {
            ViewOutputExpander::expand(&mut factory_impl, view_code, widgets_return_code, errors);
        } else {
            factory_impl.items.push(parse_quote! {
                fn init_widgets(
                    &mut self,
                    index: &relm4::factory::DynamicIndex,
                    root: &Self::Root,
                    returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
                    sender: relm4::factory::FactoryComponentSender<Self>,
                ) -> Self::Widgets {
                    #view_code
                    #widgets_return_code
                }
            });
        }

        factory_impl.items.push(parse_quote! {
            type Root = #root_widget_type;
        });

        factory_impl.items.push(parse_quote! {
            fn init_root(&self) -> Self::Root {
                #init_root
            }
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
                sender: relm4::factory::FactoryComponentSender<Self>,
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

    let outer_attrs = &factory_impl.attrs;
    let widgets_struct = factory_visitor.widgets_ty.map(|ty| {
        quote! {
            #[allow(dead_code)]
            #(#outer_attrs)*
            #[derive(Debug)]
            #vis struct #ty {
                #struct_fields
                #additional_fields
            }
        }
    });

    let errors = errors.iter().map(syn::Error::to_compile_error);

    quote! {
        #widgets_struct

        #factory_impl

        #(#errors)*
    }
}
