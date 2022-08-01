use proc_macro2::Span as Span2;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, FnArg, Ident, ImplItemMethod, Pat, Path, PathArguments, PathSegment};

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
