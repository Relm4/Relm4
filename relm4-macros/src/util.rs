use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use syn::punctuated::Punctuated;

use syn::{Ident, Path, PathArguments, PathSegment};

pub(super) fn strings_to_path(strings: &[&str]) -> Path {
    let path_segments: Vec<PathSegment> = strings
        .iter()
        .map(|string| -> PathSegment {
            PathSegment {
                ident: Ident::new(string, Span2::call_site()),
                arguments: PathArguments::None,
            }
        })
        .collect();
    Path {
        leading_colon: None,
        segments: Punctuated::from_iter(path_segments),
    }
}

pub(super) fn item_impl_error(original_input: TokenStream) -> TokenStream {
    let macro_impls = quote::quote! {
        macro_rules! view_output {
            () => { todo!() };
        }
        macro_rules! view {
            () => {};
            ($tt:tt) => {};
            ($tt:tt $($y:tt)+) => {}
        }
    }
    .into();
    vec![macro_impls, original_input].into_iter().collect()
}
