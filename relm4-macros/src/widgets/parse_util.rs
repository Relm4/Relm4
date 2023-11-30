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
        let segments = self.path.segments.iter().map(|seg| seg.ident.to_string());
        unique_ident_from_parts(segments)
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
    unique_ident_from_parts([string])
}

pub(crate) fn unique_ident_from_parts<I, T>(parts: I) -> Ident
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    let unique_number = COUNTER.fetch_add(1, Ordering::Relaxed).to_string();

    let name = parts
        .into_iter()
        .map(|part| part.as_ref().to_lowercase())
        .chain(std::iter::once(unique_number))
        .collect::<Vec<_>>()
        .join("_");

    Ident::new(&name, Span2::mixed_site())
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
