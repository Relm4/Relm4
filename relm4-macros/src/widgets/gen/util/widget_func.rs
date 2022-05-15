use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::Error;

use crate::widgets::{Widget, WidgetFunc};

impl Widget {
    /// Get tokens for the widget's type.
    pub fn func_type_token_stream(&self) -> TokenStream2 {
        let is_local = self.attr.is_local_attr();
        let func = &self.func;
        let path = &self.func.path;
        let mut tokens = TokenStream2::new();

        // If type was specified, use it
        let (type_segments, num_of_segments) = if let Some(ty) = &func.ty {
            (&ty.segments, ty.segments.len())
        } else if is_local {
            return Error::new(func.span().unwrap().into(),
                    &format!("You need to specify the type of the local variable. Use this instead: {} -> Type {{ ...", 
                    self.name)).into_compile_error();
        } else if func.args.is_some() {
            // If for example gtk::Box::new() was used, ignore ::new()
            // and use gtk::Box as type.
            let len = path.segments.len();
            if len == 0 {
                panic!("Path can't be empty");
            } else if len == 1 {
                return Error::new(func.span().unwrap().into(),
                        &format!("You need to specify a type of your function. Use this instead: {}() -> Type {{ ...",
                        path.to_token_stream())).into_compile_error();
            } else {
                (&path.segments, len - 1)
            }
        } else {
            (&path.segments, path.segments.len())
        };

        let mut seg_iter = type_segments.iter().take(num_of_segments);
        let first = if let Some(first) = seg_iter.next() {
            first
        } else {
            return Error::new(
                func.span().unwrap().into(),
                "No path segments in WidgetFunc.",
            )
            .into_compile_error();
        };
        tokens.extend(first.ident.to_token_stream());

        for segment in seg_iter {
            tokens.extend(quote! {::});
            tokens.extend(segment.ident.to_token_stream());
        }

        tokens
    }
}

impl WidgetFunc {
    /// Get the tokens of the widget's function.
    pub fn func_token_stream(&self) -> TokenStream2 {
        let WidgetFunc {
            path,
            args,
            method_chain,
            ..
        } = &self;

        let mut stream = if let Some(args) = args {
            quote! { #path(#args) }
        } else {
            quote! {
                #path::default()
            }
        };

        if let Some(method_chain) = method_chain {
            stream.extend(quote! {
                .#method_chain
            })
        }

        stream
    }
}
