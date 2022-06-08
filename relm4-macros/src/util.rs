use syn::spanned::Spanned;
use syn::{Error, FnArg, Ident, ImplItemMethod, Pat, Path};

pub mod kw {
    syn::custom_keyword!(Some);
}

pub(crate) fn default_relm4_path() -> Path {
    syn::parse_quote! { ::relm4 }
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
