use proc_macro2::Span as Span2;
use syn::{spanned::Spanned, Error, Ident, ImplItemType, Result, Type};

pub(super) struct Types {
    pub widgets: Ident,
    pub init_params: ImplItemType,
    pub input: ImplItemType,
    pub output: ImplItemType,
}

macro_rules! parse_type {
    ($lit:literal, $name:ident, $ty_name:ident, $ty:ident) => {
        if $name.is_some() {
            return Err(Error::new(
                $ty_name.span(),
                &format!("Type `{}` defined multiple times", $lit),
            ));
        }
        $name = Some($ty);
    };
}

impl Types {
    pub(super) fn new(types: Vec<ImplItemType>) -> Result<Self> {
        let mut init_params = None;
        let mut input = None;
        let mut output = None;

        let mut widgets = None;

        for ty in types {
            let ident = &ty.ident;
            if ident == "InitParams" {
                parse_type!("InitParams", init_params, ident, ty);
            } else if ident == "Input" {
                parse_type!("Input", input, ident, ty);
            } else if ident == "Output" {
                parse_type!("Output", output, ident, ty);
            } else if ident == "Widgets" {
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
                        return Err(Error::new(ty.span(), "Expected an Identfier"));
                    }
                } else {
                    return Err(Error::new(ty.span(), "Expected an Identfier"));
                }
            } else {
                return Err(Error::new(
                    ident.span(),
                    "Expected a type called `Widgets`, `InitParams`, `Input` or `Output`",
                ));
            }
        }

        let widgets =
            widgets.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Widgets`"))?;
        let init_params = init_params
            .ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `InitParams"))?;
        let output =
            output.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Ouput`"))?;
        let input =
            input.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Input`"))?;

        Ok(Self {
            widgets,
            init_params,
            output,
            input,
        })
    }
}
