use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, Ident, Path};

use crate::widgets::{AssignProperty, AssignPropertyAttr, PropertyName};

impl AssignProperty {
    pub(crate) fn conditional_assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        is_conditional: bool,
        relm4_path: &Path,
    ) {
        // If the code gen path is behind a conditional widgets, handle `watch` and `track` later.
        // Normally, those would be initialized right away, but they might need access to
        // variables from a pattern, for example `Some(variable)` so they are moved inside the
        // match arm or if expression.
        if !is_conditional
            || !matches!(
                self.attr,
                AssignPropertyAttr::Track(_) | AssignPropertyAttr::Watch
            )
        {
            self.assign_stream(stream, p_name, w_name, relm4_path);
        }
    }

    pub(crate) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        let assign_fn = p_name.assign_fn_stream(w_name, relm4_path);
        let self_assign_args = p_name.assign_args_stream(w_name);
        let span = p_name.span();

        let args = self.args.as_ref().map(|args| {
            quote! {
                , #args
            }
        });

        // Destructure tuples
        let assign = if let Expr::Tuple(tuple) = &self.expr {
            tuple.elems.to_token_stream()
        } else {
            self.expr.to_token_stream()
        };

        let (block_stream, unblock_stream) = if self.block_signals.is_empty() {
            (None, None)
        } else {
            let mut block_stream = TokenStream2::default();
            let mut unblock_stream = TokenStream2::default();
            for signal_handler in &self.block_signals {
                block_stream.extend(quote_spanned! {
                    signal_handler.span() =>
                        {
                            use #relm4_path ::WidgetRef;
                            #[allow(clippy::needless_borrow)]
                            #relm4_path ::gtk::prelude::ObjectExt::block_signal(#w_name.widget_ref(), &#signal_handler);
                        }
                });
                unblock_stream.extend(quote_spanned! {
                    signal_handler.span() =>
                        {
                            use #relm4_path ::WidgetRef;
                            #[allow(clippy::needless_borrow)]
                            #relm4_path ::gtk::prelude::ObjectExt::unblock_signal(#w_name.widget_ref(), &#signal_handler);
                        }
                });
            }
            (Some(block_stream), Some(unblock_stream))
        };

        stream.extend(match (self.optional_assign, self.iterative) {
            (false, false) => {
                quote_spanned! { span =>
                    #block_stream
                    #assign_fn(#self_assign_args #assign #args);
                    #unblock_stream
                }
            }
            (true, false) => {
                quote_spanned! {
                    span => if let Some(__p_assign) = #assign {
                        #block_stream
                        #assign_fn(#self_assign_args __p_assign #args);
                        #unblock_stream
                    }
                }
            }
            (false, true) => {
                quote_spanned! {
                    span =>
                        #block_stream
                        for __elem in #assign {
                            #assign_fn(#self_assign_args __elem #args);
                        }
                        #unblock_stream
                }
            }
            (true, true) => {
                quote_spanned! {
                    span =>
                        #block_stream
                        for __elem in #assign {
                            if let Some(__p_assign) = __elem {
                                #assign_fn(#self_assign_args __p_assign #args);
                            }
                        }
                        #unblock_stream
                }
            }
        });
    }
}
