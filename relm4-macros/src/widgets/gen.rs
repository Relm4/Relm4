/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Error};

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
            PropertyType::Expr(expr) => Some(expr.to_token_stream()),
            PropertyType::Value(lit) => Some(lit.to_token_stream()),
            PropertyType::Widget(widget) => Some(widget.widget_assignment()),
            PropertyType::Watch(tokens) => Some(tokens.to_token_stream()),
            PropertyType::Args(args) => Some(args.to_token_stream()),
            PropertyType::Track(Tracker { update_fn, .. }) => Some(update_fn.to_token_stream()),
            _ => None,
        }
    }

    fn view_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Watch(token_stream) => Some(token_stream.clone()),
            _ => None,
        }
    }

    fn connect_assign_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Connect(closure) = self {
            Some(closure.to_token_stream())
        } else {
            None
        }
    }

    fn track_tokens(&self) -> Option<(TokenStream2, TokenStream2)> {
        if let PropertyType::Track(tracker) = self {
            let update_fn = &tracker.update_fn;
            let update_stream = update_fn.to_token_stream();
            let bool_stream = tracker.bool_eqation_tokens();
            Some((bool_stream, update_stream))
        } else {
            None
        }
    }

    fn factory_expr(&self) -> Option<TokenStream2> {
        if let PropertyType::Factory(expr) = self {
            Some(expr.to_token_stream())
        } else {
            None
        }
    }

    fn component_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Component(expr) = self {
            Some(quote_spanned! { expr.span() => #expr.root_widget() })
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
                <#tokens as relm4::util::default_widgets::DefaultWidget>::default_widget()
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

    pub fn widget_assignment(&self) -> TokenStream2 {
        let w_span = self.func.span();
        let w_name = &self.name;
        let out_stream = if self.assign_as_ref {
            quote_spanned! { w_span => & #w_name}
        } else {
            quote! { #w_name}
        };
        if let Some(wrapper) = &self.wrapper {
            quote_spanned! {
                wrapper.span() => #wrapper(#out_stream)
            }
        } else {
            out_stream
        }
    }

    pub fn view_stream(&self, property_stream: &mut TokenStream2) {
        let w_name = &self.name;

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.view_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                property_stream.extend(match (prop.optional_assign, prop.iterative) {
                    (false, false) => {
                        quote_spanned! {
                            p_span => self.#w_name.#p_name(#p_assign);
                        }
                    }
                    (true, false) => {
                        quote_spanned! {
                            p_span => if let Some(__p_assign) = #p_assign {
                                self.#w_name.#p_name(__p_assign);
                            }
                        }
                    }
                    (false, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                self.#w_name.#p_name(__elem);
                            }
                        }
                    }
                    (true, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                if let Some(__p_assign) = __elem {
                                    self.#w_name.#p_name(__p_assign);
                                }
                            }
                        }
                    }
                });
            }

            let fact_assign_opt = prop.ty.factory_expr();
            if let Some(f_expr) = fact_assign_opt {
                property_stream.extend(quote! {
                    #f_expr.generate(self.#w_name, sender.clone());
                });
            }
        }
    }

    pub fn property_assign_stream(&self, property_stream: &mut TokenStream2) {
        let w_name = &self.name;

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.init_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();

                property_stream.extend(match (prop.optional_assign, prop.iterative) {
                    (false, false) => {
                        quote_spanned! {
                            p_span => #w_name.#p_name(#p_assign);
                        }
                    }
                    (true, false) => {
                        quote_spanned! {
                            p_span => if let Some(__p_assign) = #p_assign {
                                #w_name.#p_name(__p_assign);
                            }
                        }
                    }
                    (false, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                #w_name.#p_name(__elem);
                            }
                        }
                    }
                    (true, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                if let Some(__p_assign) = __elem {
                                    #w_name.#p_name(__p_assign);
                                }
                            }
                        }
                    }
                });
            }

            let fact_assign_opt = prop.ty.factory_expr();
            if let Some(f_expr) = fact_assign_opt {
                property_stream.extend(quote! {
                    #f_expr.generate(#w_name, sender.clone());
                });
            }
        }
    }

    pub fn connect_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.connect_assign_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();

                let mut clone_stream = TokenStream2::new();
                if let Some(args) = &prop.args {
                    for arg in &args.inner {
                        clone_stream.extend(quote_spanned! { arg.span() =>
                            #[allow(clippy::redundant_clone)]
                            let #arg = #arg.clone();
                        });
                    }
                }

                stream.extend(quote_spanned! {
                    p_span => {
                        #clone_stream
                        #w_name.#p_name(#p_assign);
                    }
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

    pub fn component_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.component_tokens();
            if let Some(component_tokens) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();
                stream.extend(quote_spanned! {
                    p_span =>
                        self.#w_name.#p_name(#component_tokens);
                });
            }
        }
        stream
    }
}
