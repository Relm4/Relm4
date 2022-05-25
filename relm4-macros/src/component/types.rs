use proc_macro2::Span as Span2;
use syn::spanned::Spanned;
use syn::{Error, Ident, ImplItemType, Result, Type};

pub(super) struct Types {
    pub widgets: Ident,
    pub other_types: Vec<ImplItemType>,
}

impl Types {
    pub(super) fn new(types: Vec<ImplItemType>) -> Result<Self> {
        let mut other_types = Vec::new();

        let mut widgets = None;

        for ty in types {
            let ident = &ty.ident;
            if ident == "Widgets" {
                if widgets.is_some() {
                    return Err(Error::new(
                        ident.span(),
                        "Type `Widgets` defined multiple times",
                    ));
                }
                if let Type::Path(ty_path) = &ty.ty {
                    if let Some(ident) = ty_path.path.get_ident() {
                        widgets = Some(ident.clone());
                    } else {
                        return Err(Error::new(ty.span(), "Expected an Identifier"));
                    }
                } else {
                    return Err(Error::new(ty.span(), "Expected an Identifier"));
                }
            } else if ident == "Root" {
                return Err(Error::new(
                    ident.span(),
                    "The root is already defined by the view! macro",
                ));
            } else {
                other_types.push(ty);
            }
        }

        let widgets =
            widgets.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Widgets`"))?;

        Ok(Self {
            widgets,
            other_types,
        })
    }
}
