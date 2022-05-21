use proc_macro2::Span as Span2;
use syn::punctuated::Punctuated;
use syn::token::Colon2;
use syn::{Ident, Path, PathArguments, PathSegment, Token};

pub(crate) fn default_relm4_path() -> Path {
    let relm4_path_segment = PathSegment {
        ident: Ident::new("relm4", Span2::call_site()),
        arguments: PathArguments::None,
    };

    let mut relm4_segments: Punctuated<PathSegment, Colon2> = Punctuated::new();
    relm4_segments.push(relm4_path_segment);

    Path {
        leading_colon: Some(Token![::](Span2::call_site())),
        segments: relm4_segments,
    }
}
