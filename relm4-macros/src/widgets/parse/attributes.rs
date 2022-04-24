use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Error, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Result,
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
                        } else if ident == "local_ref" {
                            Attr::LocalRef(ident.clone())
                        } else if ident == "root" {
                            Attr::Root(ident.clone())
                        } else if ident == "watch" {
                            Attr::Watch(ident.clone())
                        } else if ident == "track" {
                            Attr::Track(ident.clone(), None)
                        } else if ident == "iterate" {
                            Attr::Iterate(ident.clone())
                        } else {
                            return Err(Error::new(
                                ident.span(),
                                &format!("Unexpected attribute name `{}`.", path.to_token_stream()),
                            ));
                        }
                    } else {
                        return Err(Error::new(path.span(), "Expected identifier."));
                    }
                }
                Meta::List(list) => {
                    let MetaList { path, nested, .. } = list;
                    if let Some(ident) = path.get_ident() {
                        if ident == "block_signal" {
                            let mut signal_idents = Vec::with_capacity(nested.len());
                            for meta in nested {
                                if let NestedMeta::Meta(Meta::Path(path)) = meta {
                                    if let Some(ident) = path.get_ident() {
                                        signal_idents.push(ident.clone());
                                    } else {
                                        return Err(Error::new(
                                            path.span(),
                                            "Expected identifier.",
                                        ));
                                    }
                                } else {
                                    return Err(Error::new(
                                        ident.span(),
                                        &format!("Unexpected attribute name `{}`.", ident),
                                    ));
                                }
                            }
                            Attr::BlockSignal(ident.clone(), signal_idents)
                        } else {
                            return Err(Error::new(
                                ident.span(),
                                &format!("Unexpected attribute name `{}`.", ident),
                            ));
                        }
                    } else {
                        return Err(Error::new(path.span(), "Expected identifier."));
                    }
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
                        } else if ident == "doc" {
                            Attr::Doc(lit.into_token_stream())
                        } else {
                            return Err(Error::new(
                                ident.span(),
                                &format!("Unexpected attribute name `{}`.", ident),
                            ));
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
