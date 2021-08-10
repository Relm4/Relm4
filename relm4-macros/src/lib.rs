use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, PathArguments};

mod args;
mod attrs;
mod item_impl;
mod macros;
mod types;
mod util;
mod widgets;

use attrs::Attrs;
use item_impl::ItemImpl;
use macros::Macros;
use types::ModelTypes;

#[proc_macro_attribute]
pub fn widget(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let start = std::time::Instant::now();

    let attrs = parse_macro_input!(attributes as Attrs);
    let data = parse_macro_input!(input as ItemImpl);

    let trait_generics = if let PathArguments::AngleBracketed(generics) =
        &data.trait_.segments.last().unwrap().arguments
    {
        generics
    } else {
        panic!();
    };

    let ModelTypes {
        model,
        parent_model,
    } = match ModelTypes::new(trait_generics) {
        Ok(model) => model,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let trt = data.trait_;

    let ty = data.self_ty;

    let Macros {
        widgets,
        manual_pre_init,
        manual_init,
        manual_view,
    } = match Macros::new(&data.macros, data.brace_span.unwrap()) {
        Ok(macros) => macros,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let root_widget_name = &widgets.name;
    let root_widget_type = widgets.func.type_token_stream();

    let mut widget_list = Vec::new();
    widgets.get_widget_list(&mut widget_list);

    let mut struct_stream = TokenStream2::new();
    let mut init_stream = TokenStream2::new();
    let mut return_stream = TokenStream2::new();
    let mut property_stream = TokenStream2::new();
    let mut view_stream = TokenStream2::new();
    let mut connect_stream = TokenStream2::new();
    let mut track_stream = TokenStream2::new();
    let mut component_stream = TokenStream2::new();

    for widget in widget_list {
        let w_name = &widget.name;
        let w_ty = widget.func.type_token_stream();
        let w_span = widget.func.span();
        let w_func = widget.func.func_token_stream();

        struct_stream.extend(quote_spanned! {
            w_span => #w_name: #w_ty,
        });

        init_stream.extend(quote_spanned! {
            w_span => let #w_name = #w_func;
        });

        return_stream.extend(widget.return_stream());
        widget.property_assign_stream(&mut property_stream);
        widget.view_stream(&mut view_stream);
        connect_stream.extend(widget.connect_stream());
        track_stream.extend(widget.track_stream());
        component_stream.extend(widget.component_stream());
    }

    let impl_generics = data.impl_generics;
    let where_clause = data.where_clause;

    let out = quote! {
        #[allow(dead_code)]
        #attrs struct #ty {
            #struct_stream
        }

        impl #impl_generics #trt for #ty #where_clause {
            type Root = #root_widget_type;

            /// Initialize the UI.
            fn init_view(model: &#model, parent_widgets: &<#parent_model as ::relm4::Model>::Widgets, sender: ::gtk::glib::Sender<<#model as ::relm4::Model>::Msg>) -> Self {
                #manual_pre_init
                #init_stream
                #property_stream
                #connect_stream
                #manual_init
                Self {
                    #return_stream
                }
            }

            fn connect_components(&self, components: &<#model as ::relm4::Model>::Components) {
                #component_stream
            }

            /// Return the root widget.
            fn root_widget(&self) -> Self::Root {
                self.#root_widget_name.clone()
            }

            /// Update the view to represent the updated model.
            fn view(&mut self, model: &#model, sender: ::gtk::glib::Sender<<#model as ::relm4::Model>::Msg>) {
                #manual_view
                #view_stream
                #track_stream
            }
        }
    };

    let end = std::time::Instant::now();
    let duration = end - start;
    eprintln!("INFO: widget macro took {} ms", duration.as_millis());

    out.into()
}
