use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::Path;

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyType, Widget, WidgetAttr,
};

impl Property {
    fn init_stream(&self, stream: &mut TokenStream2, relm4_path: &Path) {
        match &self.ty {
            PropertyType::Widget(widget) => {
                widget.init_stream(stream, relm4_path);
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.init_stream(stream, relm4_path);
            }
            _ => (),
        }
    }
}

impl Properties {
    fn init_stream(&self, stream: &mut TokenStream2, relm4_path: &Path) {
        for prop in &self.properties {
            prop.init_stream(stream, relm4_path);
        }
    }
}

impl Widget {
    pub fn init_stream(&self, stream: &mut TokenStream2, relm4_path: &Path) {
        self.self_init_stream(stream);
        self.other_init_stream(stream, relm4_path);
    }

    pub fn init_root_init_streams(
        &self,
        init_root_stream: &mut TokenStream2,
        init_stream: &mut TokenStream2,
        relm4_path: &Path,
    ) {
        // Init and name as return value
        self.self_init_stream(init_root_stream);
        self.name.to_tokens(init_root_stream);

        self.other_init_stream(init_stream, relm4_path);
    }

    fn self_init_stream(&self, stream: &mut TokenStream2) {
        let mutability = &self.mutable;
        let name = &self.name;
        let func = self.func.func_token_stream();
        let span = self.name.span();

        if self.attr == WidgetAttr::None {
            stream.extend(if let Some(ty) = &self.func.ty {
                quote_spanned! {
                    span => let #mutability #name: #ty = #func;
                }
            } else {
                quote_spanned! {
                    span => let #mutability #name = #func;
                }
            });
        }
    }

    fn other_init_stream(&self, stream: &mut TokenStream2, relm4_path: &Path) {
        self.properties.init_stream(stream, relm4_path);
    }
}

impl ConditionalWidget {
    fn init_stream(&self, stream: &mut TokenStream2, relm4_path: &Path) {
        let name = &self.name;

        stream.extend(quote! {
            let #name = #relm4_path::gtk::Stack::default();
        });

        if let Some(transition) = &self.transition {
            stream.extend(quote! {
                #name.set_transition_type(#relm4_path::gtk::StackTransitionType:: #transition);
            });
        }

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.init_stream(stream, relm4_path);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.init_stream(stream, relm4_path);
                }
            }
        }
    }
}
