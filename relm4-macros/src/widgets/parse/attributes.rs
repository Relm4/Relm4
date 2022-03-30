use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Error, Lit, Meta, MetaNameValue, Result,
};

use crate::widgets::{Attr, Attrs};

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let mut inner = Vec::with_capacity(attrs.len());

        for attr in attrs {
            let meta = attr.parse_meta()?;
            inner.push(match meta {
                Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "local" {
                            Attr::Local(ident.clone())
                        } else if ident == "watch" {
                            Attr::Watch(ident.clone())
                        } else if ident == "track" {
                            Attr::Track(ident.clone(), None)
                        } else if ident == "iterate" {
                            Attr::Iterate(ident.clone())
                        } else {
                            return Err(Error::new(ident.span(), "Unexpected attribute name."));
                        }
                    } else {
                        return Err(Error::new(path.span(), "Expected identifier."));
                    }
                }
                Meta::List(list) => {
                    return Err(Error::new(list.span(), "Unexpected list attribute type."));
                }
                Meta::NameValue(name_value) => {
                    let MetaNameValue { path, lit, .. } = name_value;

                    if let Some(ident) = path.get_ident() {
                        if ident == "track" {
                            if let Lit::Str(string) = lit {
                                Attr::Track(ident.clone(), Some(string.parse()?))
                            } else {
                                return Err(Error::new(
                                    lit.span(),
                                    "Expected string attribute value.",
                                ));
                            }
                        } else {
                            return Err(Error::new(ident.span(), "Unexpected attribute name."));
                        }
                    } else {
                        return Err(Error::new(path.span(), "Expected identifier."));
                    }
                }
            });
        }

        Ok(Attrs { inner })
    }
}
