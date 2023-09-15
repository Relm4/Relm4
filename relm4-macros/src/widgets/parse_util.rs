use std::sync::atomic::{AtomicU16, Ordering};

use proc_macro2::Span as Span2;
use syn::parse::ParseBuffer;
use syn::spanned::Spanned;
use syn::{braced, bracketed, parenthesized, Error, Ident, Path};

use super::{ParseError, PropertyName};
use crate::widgets::{AssignPropertyAttr, WidgetAttr, WidgetFunc};

pub(super) fn attr_twice_error(span: Span2) -> Error {
    Error::new(span, "Cannot use the same attribute twice.")
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
        idents_to_snake_case(
            self.path.segments.iter().map(|seg| &seg.ident),
            Span2::call_site(),
        )
    }
}

impl WidgetAttr {
    pub(super) fn is_local_attr(&self) -> bool {
        matches!(self, Self::Local | Self::LocalRef)
    }
}

impl AssignPropertyAttr {
    pub(super) fn should_skip_init(&self) -> bool {
        match self {
            Self::None => false,
            Self::Watch { skip_init } | Self::Track { skip_init, .. } => skip_init.is_some(),
        }
    }
}

impl PartialEq for AssignPropertyAttr {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

pub(crate) fn string_to_snake_case(string: &str) -> Ident {
    idents_to_snake_case(
        [Ident::new(string, Span2::mixed_site())].iter(),
        Span2::mixed_site(),
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

/// Weird hack to work around syn's awkward macros
/// that always return [`syn::Error`] and are worse
/// in every aspect compared to regular Rust code.
///
/// Sadly, the regular Rust API won't be made public,
/// see [#1190](https://github.com/dtolnay/syn/issues/1190).
pub(super) fn parens<'a>(input: &'a ParseBuffer<'_>) -> Result<ParseBuffer<'a>, ParseError> {
    let content = (move || {
        let content;
        parenthesized!(content in input);
        Ok(content)
    })();
    Ok(content?)
}

/// Weird hack to work around syn's awkward macros
/// that always return [`syn::Error`] and are worse
/// in every aspect compared to regular Rust code.
///
/// Sadly, the regular Rust API won't be made public,
/// see [#1190](https://github.com/dtolnay/syn/issues/1190).
pub(super) fn braces<'a>(input: &'a ParseBuffer<'_>) -> Result<ParseBuffer<'a>, ParseError> {
    let content = (move || {
        let content;
        braced!(content in input);
        Ok(content)
    })();
    Ok(content?)
}

/// Weird hack to work around syn's awkward macros
/// that always return [`syn::Error`] and are worse
/// in every aspect compared to regular Rust code.
///
/// Sadly, the regular Rust API won't be made public,
/// see [#1190](https://github.com/dtolnay/syn/issues/1190).
pub(super) fn brackets<'a>(input: &'a ParseBuffer<'_>) -> Result<ParseBuffer<'a>, ParseError> {
    let content = (move || {
        let content;
        bracketed!(content in input);
        Ok(content)
    })();
    Ok(content?)
}
