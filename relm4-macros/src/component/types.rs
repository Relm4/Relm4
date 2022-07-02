use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::spanned::Spanned;
use syn::{Error, Ident, ImplItemType, Type};

pub(super) struct Types {
    pub widgets: Ident,
    pub other_types: Vec<ImplItemType>,
}

impl Types {
    pub(super) fn new(types: Vec<ImplItemType>) -> (Self, Option<TokenStream2>) {
        let mut other_types = Vec::new();

        let mut widgets = None;
        let mut errors = Vec::new();

        for ty in types {
            let ident = &ty.ident;
            if ident == "Widgets" {
                if widgets.is_some() {
                    errors.push(Error::new(
                        ident.span(),
                        "Type `Widgets` defined multiple times",
                    ));
                }
                if let Type::Path(ty_path) = &ty.ty {
                    if let Some(ident) = ty_path.path.get_ident() {
                        widgets = Some(ident.clone());
                    } else {
                        errors.push(Error::new(ty.span(), "Expected an Identifier"));
                    }
                } else {
                    errors.push(Error::new(ty.span(), "Expected an Identifier"));
                }
            } else if ident == "Root" {
                errors.push(Error::new(
                    ident.span(),
                    "The root is already defined by the view! macro",
                ));
            } else {
                other_types.push(ty);
            }
        }

        let widgets = if let Some(widgets) = widgets {
            widgets
        } else {
            errors.push(Error::new(
                Span2::call_site(),
                "Did not find type `Widgets`",
            ));
            Ident::new("__PlaceholderWidgetsType", Span2::call_site())
        };

        (
            Self {
                widgets,
                other_types,
            },
            if errors.is_empty() {
                None
            } else {
                let mut tokens = TokenStream2::new();
                for error in errors {
                    tokens.extend(error.to_compile_error());
                }
                Some(tokens)
            },
        )
    }
}
