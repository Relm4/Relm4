use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::Ident;

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, ParseError, Properties, Property, PropertyType,
    ReturnedWidget, Widget,
};

impl Property {
    fn error_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        match &self.ty {
            PropertyType::ParseError(error) => error.error_stream(stream, w_name),
            PropertyType::SignalHandler(_) | PropertyType::Assign(_) => (),
            PropertyType::Widget(widget) => widget.error_stream(stream),
            PropertyType::ConditionalWidget(cond_widget) => cond_widget.error_stream(stream),
        }
    }
}

impl Properties {
    fn error_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        for prop in &self.properties {
            prop.error_stream(stream, w_name);
        }
    }
}

impl ParseError {
    fn error_stream(&self, stream: &mut TokenStream2, w_name: &Ident) {
        match self {
            ParseError::Ident((ident, tokens)) => stream.extend(quote! {
                #tokens
                #w_name.#ident ;
            }),
            ParseError::Path((path, tokens)) => stream.extend(quote! {
                #tokens
                #path ;
            }),
            ParseError::Generic(generic_error) => generic_error.to_tokens(stream),
        }
    }
}

impl Widget {
    pub(crate) fn error_stream(&self, stream: &mut TokenStream2) {
        let w_name = &self.name;
        self.properties.error_stream(stream, w_name);
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.error_stream(stream);
        }
    }
}

impl ConditionalWidget {
    fn error_stream(&self, stream: &mut TokenStream2) {
        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.error_stream(stream);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.error_stream(stream)
                }
            }
        }
    }
}

impl ReturnedWidget {
    fn error_stream(&self, stream: &mut TokenStream2) {
        let w_name = &self.name;
        self.properties.error_stream(stream, w_name);
    }
}
