use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Error, Ident};

use super::{Property, PropertyName, PropertyType, Tracker, Widget, WidgetFunc};

impl PropertyType {
    fn init_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Expr(expr) => Some(expr.to_token_stream()),
            PropertyType::Value(lit) => Some(lit.to_token_stream()),
            PropertyType::Widget(widget) => Some(widget.widget_assignment()),
            PropertyType::Watch(tokens) => Some(tokens.to_token_stream()),
            PropertyType::Args(args) => Some(args.to_token_stream()),
            PropertyType::Track(Tracker { update_fns, .. }) => Some(quote! { #(#update_fns),* }),
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
            let update_fns = &tracker.update_fns;
            let update_stream = quote! { #(#update_fns),* };
            let bool_stream = tracker.bool_fn.to_token_stream();
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
            Some(quote_spanned! { expr.span() => #expr })
        } else {
            None
        }
    }

    fn connect_component_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::ConnectComponent(closure) = self {
            Some(closure.to_token_stream())
        } else {
            None
        }
    }
}

impl Property {
    fn args_stream(&self) -> TokenStream2 {
        if let Some(args) = &self.args {
            quote! { ,#args }
        } else {
            TokenStream2::new()
        }
    }
}

impl WidgetFunc {
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
                <#tokens as relm4::util::default_widgets::DefaultWidget>::default_widget()
            }
        }
    }
}

impl PropertyName {
    fn assign_fn_stream(&self, w_name: &Ident) -> TokenStream2 {
        match self {
            PropertyName::Ident(ident) => {
                quote! { #w_name.#ident }
            }
            PropertyName::Path(path) => path.to_token_stream(),
        }
    }

    fn assign_args_stream(&self, w_name: &Ident) -> Option<TokenStream2> {
        match self {
            PropertyName::Ident(_) => None,
            PropertyName::Path(_) => Some(quote! { &#w_name, }),
        }
    }

    fn self_assign_fn_stream(&self, w_name: &Ident) -> TokenStream2 {
        match self {
            PropertyName::Ident(ident) => {
                quote! { self.#w_name.#ident }
            }
            PropertyName::Path(path) => path.to_token_stream(),
        }
    }

    fn self_assign_args_stream(&self, w_name: &Ident) -> Option<TokenStream2> {
        match self {
            PropertyName::Ident(_) => None,
            PropertyName::Path(_) => Some(quote! { &self.#w_name, }),
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
                let assign_fn = prop.name.self_assign_fn_stream(w_name);
                let self_assign_args = prop.name.self_assign_args_stream(w_name);

                property_stream.extend(match (prop.optional_assign, prop.iterative) {
                    (false, false) => {
                        quote_spanned! {
                            p_span => #assign_fn(#self_assign_args #p_assign);
                        }
                    }
                    (true, false) => {
                        quote_spanned! {
                            p_span => if let Some(__p_assign) = #p_assign {
                                #assign_fn(#self_assign_args __p_assign);
                            }
                        }
                    }
                    (false, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                #assign_fn(#self_assign_args __elem);
                            }
                        }
                    }
                    (true, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                if let Some(__p_assign) = __elem {
                                    #assign_fn(#self_assign_args __p_assign);
                                }
                            }
                        }
                    }
                });
            }

            let fact_assign_opt = prop.ty.factory_expr();
            if let Some(f_expr) = fact_assign_opt {
                property_stream.extend(quote! {
                    #f_expr.generate(&self.#w_name, sender.clone());
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
                let args_stream = prop.args_stream();

                let assign_fn = prop.name.assign_fn_stream(w_name);
                let self_assign_args = prop.name.assign_args_stream(w_name);

                property_stream.extend(match (prop.optional_assign, prop.iterative) {
                    (false, false) => {
                        quote_spanned! {
                            p_span => #assign_fn(#self_assign_args #p_assign #args_stream);
                        }
                    }
                    (true, false) => {
                        quote_spanned! {
                            p_span => if let Some(__p_assign) = #p_assign {
                                #assign_fn(#self_assign_args __p_assign #args_stream);
                            }
                        }
                    }
                    (false, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                #assign_fn(#self_assign_args __elem #args_stream );
                            }
                        }
                    }
                    (true, true) => {
                        quote_spanned! {
                            p_span => for __elem in #p_assign {
                                if let Some(__p_assign) = __elem {
                                    #assign_fn(#self_assign_args __p_assign #args_stream );
                                }
                            }
                        }
                    }
                });
            }

            let fact_assign_opt = prop.ty.factory_expr();
            if let Some(f_expr) = fact_assign_opt {
                property_stream.extend(quote! {
                    #f_expr.generate(&#w_name, sender.clone());
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

                let assign_fn = prop.name.assign_fn_stream(w_name);
                let self_assign_args = prop.name.assign_args_stream(w_name);

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
                        #assign_fn(#self_assign_args #p_assign);
                    }
                });
            }
        }

        stream
    }

    pub fn connect_component_stream(&self) -> TokenStream2 {
        let w_name = &self.name;
        let mut stream = TokenStream2::new();

        for prop in &self.properties.properties {
            let p_assign_opt = prop.ty.connect_component_tokens();
            if let Some(p_assign) = p_assign_opt {
                let p_name = &prop.name;
                let p_span = p_name.span().unwrap().into();

                let assign_fn = prop.name.self_assign_fn_stream(w_name);
                let self_assign_args = prop.name.self_assign_args_stream(w_name);

                let mut arg_stream = TokenStream2::new();
                if let Some(args) = &prop.args {
                    for arg in &args.inner {
                        arg_stream.extend(quote_spanned! { arg.span() =>
                            let #arg;
                        });
                    }
                }

                stream.extend(quote_spanned! {
                    p_span => {
                        #arg_stream
                        #assign_fn(#self_assign_args #p_assign);
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

                let assign_fn = prop.name.self_assign_fn_stream(w_name);
                let self_assign_args = prop.name.self_assign_args_stream(w_name);
                let args_stream = prop.args_stream();

                stream.extend(quote_spanned! {
                    p_span =>  if #bool_stream {
                        #assign_fn(#self_assign_args #update_stream #args_stream);
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

                let args_stream = prop.args_stream();
                let assign_fn = prop.name.self_assign_fn_stream(w_name);
                let self_assign_args = prop.name.self_assign_args_stream(w_name);

                stream.extend(quote_spanned! {
                    p_span =>
                        #assign_fn(#self_assign_args #component_tokens #args_stream);
                });
            }
        }
        stream
    }
}
