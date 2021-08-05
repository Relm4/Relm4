use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, ItemImpl};

mod args;
mod attrs;
mod macros;
mod types;
mod util;
mod widgets;

use attrs::Attrs;
use macros::Macros;
use types::ModelType;

#[proc_macro_attribute]
pub fn widget(attributes: TokenStream, input: TokenStream) -> TokenStream {
    let start = std::time::Instant::now();

    let attrs = parse_macro_input!(attributes as Attrs);

    let data = parse_macro_input!(input as ItemImpl);
    let span = data.span();

    let ModelType { model } = match ModelType::new(span.unwrap(), &data.items) {
        Ok(model) => model,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let trt = match util::trait_to_path(data.self_ty.span().unwrap(), data.trait_) {
        Ok(trt) => trt,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let ty = data.self_ty;

    let Macros {
        widgets,
        manual_init,
        manual_view,
    } = match Macros::new(span.unwrap(), &data.items) {
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
    }

    if let Some(manual_init_stream) = manual_init {
        init_stream.extend(manual_init_stream);
    }
    if let Some(manual_view_stream) = manual_view {
        view_stream.extend(manual_view_stream);
    }

    let out = quote! {
        #attrs struct #ty {
            #struct_stream
        }

        impl #trt for #ty {
            type Root = #root_widget_type;
            type Model = #model;

            /// Initialize the UI.
            fn init_view(model: &Self::Model, components: &<Self::Model as ::relm4::Model>::Components, sender: ::gtk::glib::Sender<<Self::Model as ::relm4::Model>::Msg>) -> Self {
                #init_stream
                #property_stream
                #connect_stream
                Self {
                    #return_stream
                }
            }

            /// Return the root widget.
            fn root_widget(&self) -> Self::Root {
                self.#root_widget_name.clone()
            }

            /// Update the view to represent the updated model.
            fn view(&mut self, model: &Self::Model, sender: ::gtk::glib::Sender<<Self::Model as ::relm4::Model>::Msg>) {
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
