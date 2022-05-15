use syn::spanned::Spanned;
use syn::{Error, Ident, Result};

use super::PropertyName;
use crate::util;
use crate::widgets::{AssignPropertyAttr, WidgetAttr, WidgetFunc};

pub(super) fn attr_twice_error<T: Spanned>(attr: &T) -> Error {
    Error::new(attr.span(), "Cannot use the same attribute twice.")
}

impl WidgetFunc {
    pub(super) fn into_property_name(self) -> Result<PropertyName> {
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
        util::idents_to_snake_case(
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
