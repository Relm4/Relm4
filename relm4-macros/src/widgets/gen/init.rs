use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyType, Widget, WidgetAttr,
};

impl Property {
    fn init_stream(&self, stream: &mut TokenStream2) {
        match &self.ty {
            PropertyType::Widget(widget) => {
                widget.init_stream(stream);
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.init_stream(stream);
            }
            _ => (),
        }
    }
}

impl Properties {
    fn init_stream(&self, stream: &mut TokenStream2) {
        for prop in &self.properties {
            prop.init_stream(stream);
        }
    }
}

impl Widget {
    pub(crate) fn init_stream(&self, stream: &mut TokenStream2) {
        self.self_init_stream(stream);
        self.other_init_stream(stream);
    }

    pub(crate) fn init_root_init_streams(
        &self,
        init_root_stream: &mut TokenStream2,
        init_stream: &mut TokenStream2,
    ) {
        // Init and name as return value
        self.self_init_stream(init_root_stream);
        self.name.to_tokens(init_root_stream);

        self.other_init_stream(init_stream);
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

    fn other_init_stream(&self, stream: &mut TokenStream2) {
        self.properties.init_stream(stream);
    }
}

impl ConditionalWidget {
    fn init_stream(&self, stream: &mut TokenStream2) {
        let name = &self.name;
        let gtk_import = crate::gtk_import();

        stream.extend(quote_spanned! {
            name.span() =>
                let #name = #gtk_import::Stack::default();
        });

        if let Some(transition) = &self.transition {
            stream.extend(quote_spanned! {
                transition.span() =>
                    #name.set_transition_type(#gtk_import::StackTransitionType:: #transition);
            });
        }

        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.init_stream(stream);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.init_stream(stream);
                }
            }
        }
    }
}
