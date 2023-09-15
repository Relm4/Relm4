use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{spanned::Spanned, Error, Ident, ImplItem, ItemImpl, Visibility};

use crate::{
    token_streams::{TokenStreams, TraitImplDetails},
    widgets::ViewWidgets,
};

pub(crate) fn generate_tokens(vis: Option<Visibility>, mut item_impl: ItemImpl) -> TokenStream2 {
    if item_impl.items.len() != 1 {
        return Error::new(
            item_impl.span(),
            "Expected only one view macro and nothing else",
        )
        .into_compile_error();
    }

    let item = item_impl.items.pop().unwrap();

    if let ImplItem::Macro(mac) = item {
        if Some("view") == mac.mac.path.get_ident().map(|i| i.to_string()).as_deref() {
            match syn::parse::<ViewWidgets>(mac.mac.tokens.into()) {
                Ok(mut view_widgets) => {
                    view_widgets.mark_root_as_used();

                    let TokenStreams {
                        error,
                        init,
                        assign,
                        struct_fields,
                        return_fields,
                        ..
                    } = view_widgets.generate_streams(
                        &TraitImplDetails {
                            vis: vis.clone(),
                            model_name: Ident::new("_", Span2::mixed_site()),
                            sender_name: Ident::new("sender", Span2::call_site()),
                            root_name: None,
                        },
                        true,
                    );

                    let view_output = quote! {
                        #init
                        #assign
                        {
                            #error
                        }
                    };

                    let root_widget_type = view_widgets.root_type();
                    item_impl.items.push(ImplItem::Verbatim(quote! {
                        type Root = #root_widget_type;
                    }));

                    let root_name = view_widgets.root_name();

                    item_impl.items.push(ImplItem::Verbatim(quote! {
                        fn init() -> Self {
                            #view_output
                            Self {
                                #return_fields
                            }
                        }
                    }));

                    let type_name = &item_impl.self_ty;

                    quote! {
                        #[derive(Debug, Clone)]
                        #vis struct #type_name {
                            #struct_fields
                        }

                        impl ::std::convert::AsRef<#root_widget_type> for #type_name {
                            fn as_ref(&self) -> &#root_widget_type {
                                &self.#root_name
                            }
                        }

                        impl ::std::ops::Deref for #type_name {
                            type Target = #root_widget_type;

                            fn deref(&self) -> &Self::Target {
                                &self.#root_name
                            }
                        }

                        #item_impl
                    }
                }
                Err(err) => err.to_compile_error(),
            }
        } else {
            Error::new(mac.mac.path.span(), "Expected a view macro").into_compile_error()
        }
    } else {
        Error::new(item.span(), "Expected a macro").into_compile_error()
    }
}
