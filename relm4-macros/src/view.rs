use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{component, util, widgets::TopLevelWidget};

pub(super) fn generate_tokens(
    input: TokenStream,
) -> TokenStream {
    let top_level_widget = parse_macro_input!(input as TopLevelWidget);
    let relm4_path = util::default_relm4_path();

    // Use unit type
    let model_type = syn::Type::Tuple(syn::TypeTuple {
        paren_token: syn::token::Paren::default(),
        elems: syn::punctuated::Punctuated::new(),
    });

    let component::token_streams::TokenStreams {
        init,
        assign,
        connect,
        ..
    } = top_level_widget.generate_streams(&None, &model_type, &relm4_path, false);

    let output = quote! {
        #init
        #assign
        #connect
    };
    output.into()
}