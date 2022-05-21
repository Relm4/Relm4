use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::widgets::ViewWidgets;
use crate::{component, util};

pub(super) fn generate_tokens(input: TokenStream) -> TokenStream {
    let view_widgets: ViewWidgets = parse_macro_input!(input);
    let relm4_path = util::default_relm4_path();

    let component::token_streams::TokenStreams {
        init,
        assign,
        connect,
        ..
    } = view_widgets.generate_streams(&None, &relm4_path, true);

    let output = quote! {
        #init
        #connect
        #assign
    };
    output.into()
}
