use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Error};

use crate::widgets::gen::WidgetFunc;

impl WidgetFunc {
    /// Get tokens for the widget's type.
    pub fn type_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();

        // If type was specified, use it
        let segments = if let Some(ty) = &self.ty {
            &ty[..]
        } else if self.args.is_some() {
            // If for example gtk::Box::new() was used, ignore ::new()
            // and use gtk::Box as type.
            let len = self.path_segments.len();
            if len == 0 {
                return Error::new(self.span().unwrap().into(), "Expected path here.")
                    .into_compile_error();
            } else if len == 1 {
                return Error::new(self.span().unwrap().into(), &format!("You need to specify a type of your function. Use this instead: {}() -> type {{", self.path_segments.first().unwrap())).into_compile_error();
            } else {
                let last_index = len - 1;
                &self.path_segments[0..last_index]
            }
        } else {
            &self.path_segments[..]
        };

        let mut seg_iter = segments.iter();
        let first = if let Some(first) = seg_iter.next() {
            first
        } else {
            return Error::new(
                self.span().unwrap().into(),
                "No path segments in WidgetFunc.",
            )
            .into_compile_error();
        };
        tokens.extend(first.to_token_stream());

        for segment in seg_iter {
            tokens.extend(quote! {::});
            tokens.extend(segment.to_token_stream());
        }

        tokens
    }

    /// Get the tokens of the widget's function.
    pub fn func_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();

        let mut seg_iter = self.path_segments.iter();
        tokens.extend(
            seg_iter
                .next()
                .expect("No path segments in WidgetFunc. Can't generate function tokens.")
                .to_token_stream(),
        );

        for segment in seg_iter {
            tokens.extend(quote! {::});
            tokens.extend(segment.to_token_stream());
        }

        if let Some(args) = &self.args {
            tokens.extend(quote! {(#args)});
            tokens
        } else {
            quote! {
                #tokens::default()
            }
        }
    }
}
