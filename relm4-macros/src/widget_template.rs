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
            match syn::parse_macro_input::parse::<ViewWidgets>(mac.mac.tokens.into()) {
                Ok(view_widgets) => {
                    let TokenStreams {
                        error,
                        init,
                        assign,
                        connect,
                        ..
                    } = view_widgets.generate_streams(
                        &TraitImplDetails {
                            vis: None,
                            model_name: Ident::new("_", Span2::call_site()),
                            sender_name: Ident::new("sender", Span2::call_site()),
                            root_name: None,
                        },
                        true,
                    );

                    let view_output = quote! {
                        #init
                        #connect
                        #assign
                        {
                            #error
                        }
                    };

                    let root_widget_type = view_widgets.root_type();
                    item_impl.items.push(ImplItem::Verbatim(quote! {
                        type Widget = #root_widget_type;
                    }));

                    let root_name = &view_widgets
                        .top_level_widgets
                        .iter()
                        .find(|w| w.root_attr.is_some())
                        .unwrap()
                        .inner
                        .name;

                    item_impl.items.push(ImplItem::Verbatim(quote! {
                        fn init() -> Self::Widget {
                            #view_output
                            #root_name
                        }
                    }));

                    let type_name = &item_impl.self_ty;
                    quote! {
                        #vis struct #type_name;

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
