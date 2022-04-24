use syn::{spanned::Spanned, Error, Ident, Result};

use crate::util;

use crate::widgets::{
    AssignPropertyAttr, PropertyFunc, PropertyName, WidgetAttr, WidgetFunc, WidgetFuncPath,
};

pub(super) fn attr_twice_error<T: Spanned>(attr: &T) -> Error {
    Error::new(attr.span(), "Cannot use the same attribute twice.")
}

impl PropertyFunc {
    pub(super) fn into_widget_func(self) -> WidgetFunc {
        match self {
            Self::Path(path) => WidgetFunc {
                path: WidgetFuncPath::Path(path),
                ty: None,
                args: None,
            },
            Self::Ident(ident) => WidgetFunc {
                path: WidgetFuncPath::Path(ident.into()),
                ty: None,
                args: None,
            },
            Self::Func(func) => func,
        }
    }

    pub(super) fn into_property_name(self) -> Result<PropertyName> {
        match self {
            PropertyFunc::Ident(ident) => Ok(PropertyName::Ident(ident)),
            PropertyFunc::Path(path) => Ok(PropertyName::Path(path)),
            PropertyFunc::Func(func) => Err(Error::new(func.span(), "Expected a path.")),
        }
    }
}

impl WidgetFunc {
    pub(super) fn snake_case_name(&self) -> Ident {
        match &self.path {
            WidgetFuncPath::Path(path) => util::idents_to_snake_case(
                path.segments.iter().map(|seg| &seg.ident),
                self.path.span(),
            ),
            WidgetFuncPath::Method(method) => {
                util::idents_to_snake_case(method.path.iter(), self.path.span())
            }
        }
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
