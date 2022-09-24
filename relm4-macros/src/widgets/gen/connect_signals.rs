use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, Ident};

use crate::widgets::{
    ConditionalBranches, ConditionalWidget, Properties, Property, PropertyName, PropertyType,
    ReturnedWidget, SignalHandler, SignalHandlerVariant, Widget,
};

impl Property {
    fn connect_signals_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        sender_name: &Ident,
    ) {
        match &self.ty {
            PropertyType::SignalHandler(signal_handler) => {
                signal_handler.connect_signals_stream(stream, &self.name, w_name, sender_name);
            }
            PropertyType::Widget(widget) => widget.connect_signals_stream(stream, sender_name),
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.connect_signals_stream(stream, sender_name);
            }
            PropertyType::Assign(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn connect_signals_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        sender_name: &Ident,
    ) {
        for prop in &self.properties {
            prop.connect_signals_stream(stream, w_name, sender_name);
        }
    }
}

impl Widget {
    pub fn connect_signals_stream(&self, stream: &mut TokenStream2, sender_name: &Ident) {
        let w_name = &self.name;
        self.properties
            .connect_signals_stream(stream, w_name, sender_name);
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.connect_signals_stream(stream, sender_name);
        }
    }
}

impl ConditionalWidget {
    fn connect_signals_stream(&self, stream: &mut TokenStream2, sender_name: &Ident) {
        match &self.branches {
            ConditionalBranches::If(if_branches) => {
                for branch in if_branches {
                    branch.widget.connect_signals_stream(stream, sender_name);
                }
            }
            ConditionalBranches::Match((_, _, match_arms)) => {
                for arm in match_arms {
                    arm.widget.connect_signals_stream(stream, sender_name);
                }
            }
        }
    }
}

impl ReturnedWidget {
    fn connect_signals_stream(&self, stream: &mut TokenStream2, sender_name: &Ident) {
        let w_name = &self.name;
        self.properties
            .connect_signals_stream(stream, w_name, sender_name);
    }
}

impl SignalHandler {
    fn connect_signals_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        sender_name: &Ident,
    ) {
        let span = p_name.span();
        let assign_fn = p_name.assign_fn_stream(w_name);
        let self_assign_args = p_name.assign_args_stream(w_name);

        let (clone_stream, assignment) = match &self.inner {
            SignalHandlerVariant::Expr(expr) => (
                quote_spanned! {
                    span => let sender = #sender_name.clone();
                },
                quote_spanned! {
                    span => move |_| {
                        sender.input(#expr)
                    }
                },
            ),
            SignalHandlerVariant::Closure(inner) => {
                let mut clone_stream = TokenStream2::new();
                if let Some(args) = &inner.args {
                    for arg in &args.inner {
                        if let Expr::Path(path) = arg {
                            if let Some(ident) = path.path.get_ident() {
                                // Just an ident was used. Simply clone it.
                                clone_stream.extend(quote_spanned! { arg.span() =>
                                    #[allow(clippy::redundant_clone)]
                                    #[allow(clippy::clone_on_copy)]
                                    let #ident = #ident.clone();
                                });
                                continue;
                            }
                        }
                        // Allow more complex expressions such as `value = data.sender()`
                        clone_stream.extend(quote_spanned! { arg.span() =>
                            #[allow(clippy::redundant_clone)]
                            #[allow(clippy::clone_on_copy)]
                            let #arg;
                        });
                    }
                }
                (clone_stream, inner.closure.to_token_stream())
            }
        };

        stream.extend(if let Some(signal_handler_id) = &self.handler_id {
            quote_spanned! {
                span => let #signal_handler_id = {
                    #clone_stream
                    #assign_fn(#self_assign_args #assignment)
                };
            }
        } else {
            quote_spanned! {
                span => {
                    #clone_stream
                    #assign_fn(#self_assign_args #assignment);
                }
            }
        });
    }
}
