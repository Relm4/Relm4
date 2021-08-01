use proc_macro::Span;
use syn::{Error, ImplItem, Result, Type};

const MODEL_IDENT: &str = "Model";
const COMPONENTS_IDENT: &str = "Components";
const MSG_IDNET: &str = "Msg";

#[derive(Debug)]
pub(super) struct GenericTypes {
    pub model: Type,
    pub components: Type,
    pub msg: Type,
}

impl GenericTypes {
    pub fn new(span: Span, items: &[ImplItem]) -> Result<Self> {
        let mut model = None;
        let mut components = None;
        let mut msg = None;

        for item in items {
            if let ImplItem::Type(ty) = item {
                let ident = &ty.ident;
                if ident == MODEL_IDENT {
                    model = Some(ty.ty.clone());
                } else if ident == COMPONENTS_IDENT {
                    components = Some(ty.ty.clone());
                } else if ident == MSG_IDNET {
                    msg = Some(ty.ty.clone());
                } else {
                    return Err(Error::new(
                        ident.span().unwrap().into(),
                        format!("Unknown type identifier {:?}", ident),
                    ));
                }
            }
        }

        Ok(GenericTypes {
            model: model
                .ok_or_else(|| Error::new(span.into(), "Type Model need's to be defined"))?,
            components: components
                .ok_or_else(|| Error::new(span.into(), "Type Components need's to be defined"))?,
            msg: msg.ok_or_else(|| Error::new(span.into(), "Type Msg need's to be defined"))?,
        })
    }
}
