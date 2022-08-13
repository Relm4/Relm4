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
