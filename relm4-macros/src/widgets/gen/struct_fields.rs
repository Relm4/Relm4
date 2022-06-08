use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{Path, Visibility};

use super::{ReturnedWidget, Widget};
use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyType, SignalHandler,
};

impl Property {
    fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        match &self.ty {
            PropertyType::Widget(widget) => widget.struct_fields_stream(stream, vis, relm4_path),
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.struct_fields_stream(stream, vis, relm4_path)
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.struct_fields_stream(stream, vis, relm4_path)
            }
            PropertyType::Assign(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        for prop in &self.properties {
            prop.struct_fields_stream(stream, vis, relm4_path);
        }
    }
}

impl Widget {
    pub fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
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

        self.properties
            .struct_fields_stream(stream, vis, relm4_path);
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.struct_fields_stream(stream, vis, relm4_path);
        }
    }
}

impl ConditionalWidget {
    fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        let name = &self.name;

        stream.extend(if let Some(docs) = &self.doc_attr {
            quote_spanned! {
                name.span() =>
                   #[doc = #docs]
                   #vis #name: #relm4_path::gtk::Stack,
            }
        } else {
            quote_spanned! {
                name.span() =>
                    #[allow(missing_docs)]
                    #vis #name: #relm4_path::gtk::Stack,
            }
        });

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.struct_fields_stream(stream, vis, relm4_path);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.struct_fields_stream(stream, vis, relm4_path);
                }
            }
        }
    }
}

impl ReturnedWidget {
    fn struct_fields_stream(
        &self,
        stream: &mut TokenStream2,
        vis: &Option<Visibility>,
        relm4_path: &Path,
    ) {
        if let Some(ty) = &self.ty {
            let name = &self.name;
            stream.extend(quote! {
                #[allow(missing_docs)]
                #vis #name: #ty,
            });
        }
        self.properties
            .struct_fields_stream(stream, vis, relm4_path);
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
            stream.extend(quote_spanned! {
                signal_handler_id.span() =>
                    #[allow(missing_docs)]
                    #vis #signal_handler_id: #relm4_path::gtk::glib::signal::SignalHandlerId,
            });
        }
    }
}
