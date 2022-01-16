use proc_macro2::Span as Span2;
use syn::{spanned::Spanned, Error, Ident, ImplItemType, Result, Type};

pub(super) struct Types {
    pub factory: ImplItemType,
    pub widget: Ident,
    pub view: ImplItemType,
    pub msg: ImplItemType,
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
        let mut factory = None;
        let mut view = None;
        let mut msg = None;

        let mut widget = None;

        for ty in types {
            let ident = &ty.ident;
            if ident == "Factory" {
                parse_type!("Factory", factory, ident, ty);
            } else if ident == "View" {
                parse_type!("View", view, ident, ty);
            } else if ident == "Msg" {
                parse_type!("Msg", msg, ident, ty);
            } else if ident == "Widgets" {
                if widget.is_some() {
                    return Err(Error::new(
                        ident.span(),
                        "Type `Widgets` defined multiple times",
                    ));
                }
                if let Type::Path(ty_path) = &ty.ty {
                    if let Some(ident) = ty_path.path.get_ident() {
                        widget = Some(ident.clone());
                    } else {
                        return Err(Error::new(ty.span(), "Expected an Identfier"));
                    }
                } else {
                    return Err(Error::new(ty.span(), "Expected an Identfier"));
                }
            } else {
                return Err(Error::new(ident.span(), "Expected a type called `Factory`, `View`, `Msg` or `Widgets`"));
            }
        }

        let factory =
            factory.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Factory`"))?;
        let view =
            view.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `View`"))?;
        let msg = msg.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Msg`"))?;
        let widget =
            widget.ok_or_else(|| Error::new(Span2::call_site(), "Did not find type `Widgets`"))?;

        Ok(Self {
            factory,
            view,
            msg,
            widget,
        })
    }
}
