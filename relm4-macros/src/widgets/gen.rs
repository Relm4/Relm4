use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, *};

use super::{PropertyType, Tracker, Widget, WidgetFunc};

impl Tracker {
    fn bool_eqation_tokens(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();

        let mut iter = self.items.iter();
        let first = iter.next().expect("No items to be tracked");
        tokens.extend(quote_spanned! { first.span() => #first });

        for item in iter {
            tokens.extend(quote_spanned! { item.span() => || #item });
        }

        tokens
    }
}

impl PropertyType {
    fn init_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Expr(expr) => Some(quote_spanned! { expr.span() => #expr }),
            PropertyType::Value(lit) => Some(quote_spanned! { lit.span() => #lit }),
            PropertyType::Widget(widget) => Some(widget.property_assignment()),
            PropertyType::Watch(tokens) => Some(quote_spanned! { tokens.span() => #tokens }),
            PropertyType::Args(args) => Some(args.to_token_stream()),
            PropertyType::Track(Tracker { update_fn, .. }) => {
                Some(quote_spanned! { update_fn.span() => #update_fn })
            }
            PropertyType::Component(expr) => Some(quote_spanned! { expr.span() => #expr.root_widget()}),
            _ => None,
        }
    }

    fn view_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Watch(token_stream) => {
                Some(quote_spanned! { token_stream.span() => #token_stream })
            }
            _ => None,
        }
    }

    fn connect_assign_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Connect(closure) = self {
            Some(quote_spanned! { closure.span() => {
                #[allow(dead_code)]
                #[allow(clippy::redundant_clone)]
                let sender = sender.clone();
                #closure
            }})
        } else {
            None
        }
    }

    fn track_tokens(&self) -> Option<(TokenStream2, TokenStream2)> {
        if let PropertyType::Track(tracker) = self {
            let update_fn = &tracker.update_fn;
            let update_stream = quote_spanned! { update_fn.span() => #update_fn };
            let bool_stream = tracker.bool_eqation_tokens();
            Some((bool_stream, update_stream))
        } else {
            None
        }
    }
}

impl WidgetFunc {
    pub fn type_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();
        let segments = if let Some(ty) = &self.ty {
            &ty[..]
        } else if self.args.is_some() {
            let len = self.path_segments.len();
            if len < 1 {
                return Error::new(self.span().unwrap().into(), "TODO").into_compile_error();
            }
            let last_index = len - 1;
            &self.path_segments[0..last_index]
        } else {
            &self.path_segments[..]
        };

        let mut seg_iter = segments.iter();
        tokens.extend(
            seg_iter
                .next()
                .expect("No path segments in WidgetFunc")
                .to_token_stream(),
        );

        for segment in seg_iter {
            tokens.extend(quote! {::});
            tokens.extend(segment.to_token_stream());
        }

        tokens
    }

    pub fn func_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();

        let mut seg_iter = self.path_segments.iter();
        tokens.extend(
            seg_iter
                .next()
                .expect("No path segments in WidgetFunc")
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
                <#tokens as relm4::default_widgets::DefaultWidget>::default_widget()
            }
        }
    }
}

impl Widget {
    pub fn return_stream(&self) -> TokenStream2 {
        let w_span = self.func.span();
        let w_name = &self.name;
        quote_spanned! {
            w_span => #w_name,
        }
    }

    pub fn property_assignment(&self) -> TokenStream2 {
        let w_span = self.func.span();
        let w_name = &self.name;
        let out_stream = if self.assign_as_ref {
            quote_spanned! { w_span => & #w_name}
        } else {
            quote_spanned! { w_span => #w_name}
        };
        if let Some(wrapper) = &self.wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }

    pub fn view_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut property_stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.view_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                property_stream.extend(quote_spanned! {
                    p_span => self.#w_name.#p_name(#p_assign);
                });
            }
        }

        property_stream
    }

    pub fn property_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut property_stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.init_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                property_stream.extend(quote_spanned! {
                    p_span => #w_name.#p_name(#p_assign);
                });
            }
        }

        property_stream
    }

    pub fn connect_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.connect_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                stream.extend(quote_spanned! {
                    p_span => #w_name.#p_name(#p_assign);
                });
            }
        }

        stream
    }

    pub fn track_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.track_tokens();
            if let Some((bool_stream, update_stream)) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                stream.extend(quote_spanned! {
                    p_span =>  if #bool_stream {
                        self.#w_name.#p_name(#update_stream);
                }});
            }
        }
        stream
    }
}
