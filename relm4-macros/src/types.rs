use proc_macro::Span;
use syn::{Error, ImplItem, Result, Type};

const MODEL_IDENT: &str = "Model";

#[derive(Debug)]
pub(super) struct ModelType {
    pub model: Type,
}

impl ModelType {
    pub fn new(span: Span, items: &[ImplItem]) -> Result<Self> {
        let mut model = None;

        for item in items {
            if let ImplItem::Type(ty) = item {
                let ident = &ty.ident;
                if ident == MODEL_IDENT {
                    model = Some(ty.ty.clone());
                } else {
                    return Err(Error::new(
                        ident.span().unwrap().into(),
                        format!("Unknown type identifier {:?}", ident),
                    ));
                }
            }
        }

        Ok(ModelType {
            model: model
                .ok_or_else(|| Error::new(span.into(), "Type Model needs to be defined"))?,
        })
    }
}
