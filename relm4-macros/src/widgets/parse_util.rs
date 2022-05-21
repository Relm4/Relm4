use std::sync::atomic::{AtomicU16, Ordering};

use proc_macro2::Span as Span2;
use syn::group::{parse_braces, parse_brackets, parse_parens};
use syn::parse::ParseBuffer;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Error, Ident, Path, PathArguments, PathSegment};

use super::{ParseError, PropertyName};
use crate::widgets::{parse_util, AssignPropertyAttr, WidgetAttr, WidgetFunc};

pub(super) fn attr_twice_error<T: Spanned>(attr: &T) -> Error {
    Error::new(attr.span(), "Cannot use the same attribute twice.")
}

impl From<Error> for ParseError {
    fn from(error: Error) -> Self {
        Self::Generic(error.to_compile_error())
    }
}

impl ParseError {
    pub(super) fn add_path(self, path: &Path) -> Self {
        if let ParseError::Generic(tokens) = self {
            if let Some(ident) = path.get_ident() {
                ParseError::Ident((ident.clone(), tokens))
            } else {
                ParseError::Path((path.clone(), tokens))
            }
        } else {
            self
        }
    }
}

impl WidgetFunc {
    pub(super) fn into_property_name(self) -> Result<PropertyName, Error> {
        if let Some(methods) = &self.method_chain {
            Err(Error::new(
                methods.span(),
                "Can't use method calls in property assignments",
            ))
        } else if let Some(args) = &self.args {
            Err(Error::new(
                args.span(),
                "Can't use function arguments in property assignments",
            ))
        } else {
            Ok(if let Some(ident) = self.path.get_ident() {
                PropertyName::Ident(ident.clone())
            } else {
                PropertyName::Path(self.path)
            })
        }
    }
}

impl WidgetFunc {
    pub(super) fn snake_case_name(&self) -> Ident {
        parse_util::idents_to_snake_case(
            self.path.segments.iter().map(|seg| &seg.ident),
            self.path.span(),
        )
    }
}

impl WidgetAttr {
    pub(super) fn is_local_attr(&self) -> bool {
        matches!(self, Self::Local | Self::LocalRef)
    }
}

impl PartialEq for AssignPropertyAttr {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub(crate) fn string_to_snake_case(string: &str) -> Ident {
    idents_to_snake_case(
        [Ident::new(string, Span2::call_site())].iter(),
        Span2::call_site(),
    )
}

pub(crate) fn idents_to_snake_case<'a, I: Iterator<Item = &'a Ident>>(
    idents: I,
    span: Span2,
) -> Ident {
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    let val = COUNTER.fetch_add(1, Ordering::Relaxed);
    let index_str = val.to_string();

    let segments: Vec<String> = idents
        .map(|ident| ident.to_string().to_lowercase())
        .collect();
    let length: usize =
        segments.iter().map(|seg| seg.len() + 1).sum::<usize>() + index_str.len() + 1;
    let mut name: String = String::with_capacity(length);

    for seg in &segments {
        name.push('_');
        name.push_str(seg);
    }
    name.push('_');
    name.push_str(&index_str);

    Ident::new(&name, span)
}

pub(super) fn parens<'a>(input: &'a ParseBuffer) -> Result<ParseBuffer<'a>, ParseError> {
    match parse_parens(input) {
        Ok(parens) => Ok(parens.content),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn braces<'a>(input: &'a ParseBuffer) -> Result<ParseBuffer<'a>, ParseError> {
    match parse_braces(input) {
        Ok(parens) => Ok(parens.content),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn brackets<'a>(input: &'a ParseBuffer) -> Result<ParseBuffer<'a>, ParseError> {
    match parse_brackets(input) {
        Ok(parens) => Ok(parens.content),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn strings_to_path(strings: &[&str]) -> Path {
    let path_segments: Vec<PathSegment> = strings
        .iter()
        .map(|string| PathSegment {
            ident: Ident::new(string, Span2::call_site()),
            arguments: PathArguments::None,
        })
        .collect();
    Path {
        leading_colon: None,
        segments: Punctuated::from_iter(path_segments),
    }
}
