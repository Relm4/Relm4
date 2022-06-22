use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, Ident};

use crate::component;
use crate::widgets::ViewWidgets;

pub(super) fn generate_tokens(input: TokenStream) -> TokenStream {
    let view_widgets: ViewWidgets = parse_macro_input!(input);
    let model_name = Ident::new("_", Span2::call_site());

    let component::token_streams::TokenStreams {
        error,
        init,
        assign,
        connect,
        ..
    } = view_widgets.generate_streams(&None, &model_name, None, true);

    let output = quote! {
        #init
        #connect
        #assign
        {
            #error
        }
    };
    output.into()
}
