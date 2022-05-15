use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Path};

use crate::widgets::{Property, PropertyName, PropertyType, SignalHandler};

impl Property {
    pub fn connect_signals_stream(
        &self,
        stream: &mut TokenStream2,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        if let PropertyType::SignalHandler(signal_handler) = &self.ty {
            signal_handler.connect_signals_stream(stream, &self.name, w_name, relm4_path);
        }
    }
}

impl SignalHandler {
    fn connect_signals_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        relm4_path: &Path,
    ) {
        let assign_fn = p_name.assign_fn_stream(w_name, relm4_path);
        let self_assign_args = p_name.assign_args_stream(w_name);
        let assign = &self.closure;
        let span = p_name.span();

        let mut clone_stream = TokenStream2::new();
        if let Some(args) = &self.args {
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

        stream.extend(if let Some(signal_handler_id) = &self.handler_id {
            quote_spanned! {
                span => let #signal_handler_id = {
                    #clone_stream
                    #assign_fn(#self_assign_args #assign)
                };
            }
        } else {
            quote_spanned! {
                span => {
                    #clone_stream
                    #assign_fn(#self_assign_args #assign);
                }
            }
        });
    }
}
