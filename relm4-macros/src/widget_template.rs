use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use quote::quote;
use syn::{Error, Ident, ImplItem, ItemImpl, Visibility, spanned::Spanned};

use crate::{
    token_streams::{TokenStreams, TraitImplDetails},
    widgets::ViewWidgets,
};

pub(crate) fn generate_tokens(vis: Option<Visibility>, item_impl: ItemImpl) -> TokenStream2 {
    let self_ty = item_impl.self_ty.clone();
    match try_generate_tokens(vis, item_impl) {
        Ok(tokens) => tokens,
        Err(err) => {
            let err = err.to_compile_error();
            quote! {
                #[derive(Debug, Clone)]
                pub struct #self_ty;

                impl ::std::convert::AsRef<()> for #self_ty {
                    fn as_ref(&self) -> &() {
                        todo!()
                    }
                }

                impl ::std::ops::Deref for #self_ty {
                    type Target = ();

                    fn deref(&self) -> &Self::Target {
                        &()
                    }
                }

                impl WidgetTemplate for #self_ty {
                    type Root = ();
                    type Init = ();

                    fn init(Self::Init) -> Self {
                        todo!()
                    }
                }
                #err
            }
        }
    }
}

fn try_generate_tokens(
    vis: Option<Visibility>,
    mut item_impl: ItemImpl,
) -> syn::Result<TokenStream2> {
    let mut init_type_set = false;
    let mut view_macro_idx = None;
    for (idx, item) in item_impl.items.iter().enumerate() {
        match item {
            ImplItem::Type(ty) => {
                if ty.ident == "Init" {
                    init_type_set = true;
                }
            }
            ImplItem::Macro(mac) => {
                if mac.mac.path.get_ident().map(|ident| ident == "view") == Some(true) {
                    view_macro_idx = Some(idx);
                }
            }
            _ => return Err(Error::new(item.span(), "Expected a view macro")),
        }
    }

    if !init_type_set {
        item_impl.items.push(ImplItem::Type(syn::parse_quote! {
            type Init = ();
        }));
    }

    if let Some(view_macro_idx) = view_macro_idx {
        let ImplItem::Macro(view_macro) = item_impl.items.remove(view_macro_idx) else {
            unreachable!()
        };

        let mut view_widgets = syn::parse::<ViewWidgets>(view_macro.mac.tokens.into())?;
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
            fn init(init: Self::Init) -> Self {
                #view_output
                Self {
                    #return_fields
                }
            }
        }));

        let type_name = &item_impl.self_ty;

        Ok(quote! {
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
        })
    } else {
        Err(Error::new(item_impl.span(), "Expected a view macro"))
    }
}
