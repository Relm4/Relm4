use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Path, Visibility};

use crate::widgets::{Property, PropertyType, SignalHandler};

use super::{ReturnedWidget, Widget};

impl Property {
    pub fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        match &self.ty {
            PropertyType::Assign(_) => (),
            PropertyType::Widget(widget) => widget.struct_fields_stream(stream, vis),
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.struct_fields_stream(stream, vis, relm4_path)
            }
        }
    }
}

impl Widget {
    pub fn struct_fields_stream(&self, stream: &mut TokenStream2, vis: &Option<Visibility>) {
        let name = &self.name;
        let ty = self.func_type_token_stream();

        stream.extend(if let Some(docs) = &self.doc_attr {
            quote! {
                #[doc = #docs]
                #vis #name: #ty,
            }
        } else {
            quote! {
                #[allow(missing_docs)]
                #vis #name: #ty,
            }
        });
    }
}

impl ReturnedWidget {
    pub fn struct_fields_stream(&self, stream: &mut TokenStream2, vis: &Option<Visibility>) {
        if let Some(ty) = &self.ty {
            let name = &self.name;
            stream.extend(quote! {
                #[allow(missing_docs)]
                #vis #name: #ty,
            });
        }
    }
}

impl SignalHandler {
    fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        if let Some(signal_handler_id) = &self.handler_id {
            stream.extend(quote! {
                #[allow(missing_docs)]
                #vis #signal_handler_id: #relm4_path::gtk::glib::signal::SignalHandlerId,
            });
        }
    }
}
