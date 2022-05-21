use proc_macro2::Span as Span2;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Colon2;
use syn::{Error, FnArg, Ident, ImplItemMethod, Pat, Path, PathArguments, PathSegment, Token};

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

pub(crate) fn get_ident_of_nth_func_input(
    func: &ImplItemMethod,
    index: usize,
) -> Result<Ident, Error> {
    let input = func.sig.inputs.iter().nth(index).ok_or_else(|| {
        Error::new(
            func.span().unwrap().into(),
            "`init` method must have three parameters",
        )
    })?;
    match input {
        FnArg::Receiver(recv) => Err(Error::new(recv.span(), "Expected type parameter")),
        FnArg::Typed(pat_type) => {
            if let Pat::Ident(ident) = &*pat_type.pat {
                Ok(ident.ident.clone())
            } else {
                Err(Error::new(pat_type.span(), "Expected type identifier"))
            }
        }
    }
}
