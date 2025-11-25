use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote_spanned};
use syn::Expr;
use syn::{Ident, spanned::Spanned};

use crate::widgets::{PropertyName, SignalHandler, SignalHandlerVariant};

use super::AssignInfo;

impl SignalHandler {
    pub(super) fn connect_signals_stream(
        &self,
        info: &mut AssignInfo<'_>,
        p_name: &PropertyName,
        sender_name: &Ident,
    ) {
        let span = p_name.span();
        let assign_fn = p_name.assign_fn_stream(info);
        let self_assign_args = p_name.assign_args_stream(info.widget_name);

        let (clone_stream, assignment) = match &self.inner {
            SignalHandlerVariant::Expr(expr) => (
                quote_spanned! { span =>
                    #[allow(clippy::redundant_clone)]
                    let sender = #sender_name.clone();
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
                        if let Expr::Path(path) = arg
                            && let Some(ident) = path.path.get_ident()
                        {
                            // Just an ident was used. Simply clone it.
                            clone_stream.extend(quote_spanned! { arg.span() =>
                                #[allow(clippy::redundant_clone)]
                                #[allow(clippy::clone_on_copy)]
                                let #ident = #ident.clone();
                            });
                            continue;
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

        info.stream
            .extend(if let Some(signal_handler_id) = &self.handler_id {
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
