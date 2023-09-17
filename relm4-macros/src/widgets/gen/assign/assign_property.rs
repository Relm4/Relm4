use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::Expr;

use crate::widgets::{AssignProperty, AssignPropertyAttr, PropertyName};

use super::AssignInfo;

impl AssignProperty {
    pub(crate) fn conditional_assign_stream(
        &self,
        info: &mut AssignInfo<'_>,
        p_name: &PropertyName,
        init: bool,
    ) {
        // If the code gen path is behind a conditional widgets, handle `watch` and `track` later.
        // Normally, those would be initialized right away, but they might need access to
        // variables from a pattern, for example `Some(variable)` so they are moved inside the
        // match arm or if expression.
        if !info.is_conditional
            || !matches!(
                self.attr,
                AssignPropertyAttr::Track { .. } | AssignPropertyAttr::Watch { .. }
            )
        {
            self.assign_stream(info, p_name, init);
        }
    }

    pub(crate) fn assign_stream(
        &self,
        info: &mut AssignInfo<'_>,
        p_name: &PropertyName,
        init: bool,
    ) {
        if init && self.attr.should_skip_init() {
            return;
        }

        let assign_fn = p_name.assign_fn_stream(info);
        let self_assign_args = p_name.assign_args_stream(info.widget_name);
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

        let chain = self.chain.as_ref().map(|chain| {
            quote_spanned! {
                chain.span() => .#chain
            }
        });

        let (block_stream, unblock_stream) = if init || self.block_signals.is_empty() {
            (None, None)
        } else {
            let mut block_stream = TokenStream2::default();
            let mut unblock_stream = TokenStream2::default();
            let gtk_import = crate::gtk_import();

            let w_name = info.widget_name;
            for signal_handler in &self.block_signals {
                block_stream.extend(quote_spanned! {
                    signal_handler.span() =>
                        {
                            use relm4::WidgetRef;
                            #[allow(clippy::needless_borrow)]
                            #gtk_import::prelude::ObjectExt::block_signal(#w_name.widget_ref(), &#signal_handler);
                        }
                });
                unblock_stream.extend(quote_spanned! {
                    signal_handler.span() =>
                        {
                            use relm4::WidgetRef;
                            #[allow(clippy::needless_borrow)]
                            #gtk_import::prelude::ObjectExt::unblock_signal(#w_name.widget_ref(), &#signal_handler);
                        }
                });
            }
            (Some(block_stream), Some(unblock_stream))
        };

        info.stream
            .extend(match (self.optional_assign, self.iterative) {
                (false, false) => {
                    quote_spanned! { span =>
                        #block_stream
                        #assign_fn(#self_assign_args #assign #args) #chain;
                        #unblock_stream
                    }
                }
                (true, false) => {
                    quote! {
                        if let Some(assign) = #assign {
                            #block_stream
                            #assign_fn(#self_assign_args assign #args) #chain;
                            #unblock_stream
                        }
                    }
                }
                (false, true) => {
                    quote! {
                        #block_stream
                        for elem in #assign {
                            #assign_fn(#self_assign_args elem #args) #chain;
                        }
                        #unblock_stream
                    }
                }
                (true, true) => {
                    quote_spanned! {
                        span =>
                            #block_stream
                            for elem in #assign {
                                if let Some(assign) = elem {
                                    #assign_fn(#self_assign_args assign #args) #chain;
                                }
                            }
                            #unblock_stream
                    }
                }
            });
    }
}
