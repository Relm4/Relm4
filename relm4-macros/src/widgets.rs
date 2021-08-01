use proc_macro::{self, Span};
use proc_macro2::Span as Span2;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    *,
};

use crate::util;

#[derive(Debug)]
pub(super) struct Tracker {
    items: Vec<Expr>,
    update_fn: Expr,
}

/*impl ToTokens for TrackItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            TrackItem::Expr(expr) => {
                quote_spanned! {
                    expr.span() => #expr
                }
            },
            TrackItem::Ident(ident) => {
                quote_spanned! {
                    ident.span() => model.#ident
                }
            }
        });
    }
}*/

impl Parse for Tracker {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = vec![input.parse()?];

        while !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            items.push(input.parse()?);
            /*if input.peek2(Token! [,]) {
                items.push(input.parse().map(TrackItem::Ident)?);
            } else {
                items.push(input.parse().map(TrackItem::Expr)?);
            }*/
        }

        let update_fn = if let Some(item) = items.pop() {
            /*match item {
                TrackItem::Expr(expr) => Ok(expr),
                TrackItem::Ident(ident) => Err(Error::new(ident.span(), ""))
            }*/
            Ok(item)
        } else {
            Err(input.error("Expected identifier or expression"))
        }?;

        if items.is_empty() {
            return Err(input.error("Expected at least two arguments"));
        }

        Ok(Tracker { items, update_fn })
    }
}

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

#[derive(Debug)]
pub(super) enum PropertyType {
    Expr(Expr),
    Value(Lit),
    Track(Tracker),
    Connect(ExprClosure),
    Watch(TokenStream2),
    Widget(Widget),
}

impl PropertyType {
    fn init_assign_tokens(&self) -> Option<TokenStream2> {
        match self {
            PropertyType::Expr(expr) => Some(quote_spanned! { expr.span() => #expr }),
            PropertyType::Value(lit) => Some(quote_spanned! { lit.span() => #lit }),
            PropertyType::Widget(widget) => Some(widget.property_assignment()),
            PropertyType::Watch(tokens) => Some(quote_spanned! { tokens.span() => #tokens }),
            PropertyType::Track(Tracker { update_fn, .. }) => {
                Some(quote_spanned! { update_fn.span() => #update_fn })
            }
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
            Some(quote_spanned! { closure.span() => #closure})
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

/*impl Spanned for PropertyType {
    fn span(&self) -> Span2 {
        match self {
            PropertyType::Expr(expr) => expr.span(),
            PropertyType::Ident(ident) => ident.span(),
            PropertyType::Value(value) => value.span(),
            PropertyType::Widget(widget) => widget.name.span(),
            PropertyType::Watch(tokens) => tokens.span(),
            PropertyType::Connect(closure) => closure.span(),
            _ => panic!("Ahh"),
        }
    }
}*/

#[derive(Debug)]
pub(super) struct Property {
    pub name: Ident,
    pub ty: PropertyType,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        //dbg!(&name);

        let ty = if input.peek(Token! [=>]) {
            let _arrow: Token![=>] = input.parse()?;
            input.parse().map(PropertyType::Connect)?
        } else if input.peek(Token![=]) || input.peek3(Token![=]) {
            if input.peek(Token![=]) {
                let _token: Token![=] = input.parse()?;
            } else {
                let _colon: Token! [:] = input.parse()?;
            }
            input.parse().map(PropertyType::Widget)?
        } else if input.peek(Token! [:]) {
            let _colon: Token! [:] = input.parse()?;
            if input.peek(Lit) {
                input.parse().map(PropertyType::Value)?
            } else if input.peek2(Token![!]) {
                let mac: Macro = input.parse()?;
                let segs = &mac.path.segments;

                if segs.len() == 1 {
                    let ident = &segs.first().expect("Macro has no segments").ident;

                    if ident == "track" {
                        let tokens = mac.tokens.into();
                        PropertyType::Track(parse_macro_input::parse(tokens)?)
                    } else if ident == "watch" {
                        PropertyType::Watch(mac.tokens)
                    } else {
                        input.parse().map(PropertyType::Expr)?
                    }
                } else {
                    input.parse().map(PropertyType::Expr)?
                }
            } else {
                input.parse().map(PropertyType::Expr)?
            }
        } else {
            return Err(input.error("TODO"));
        };

        Ok(Property { name, ty })
    }
}

/*impl Property {
    fn init_assignment_stream(&self) -> TokenStream2 {
        let name = self.ty.init_assign_tokens();
        let span = self.ty.span();
        quote_spanned! {
            span => #name
        }
    }

    fn view_assignment_stream(&self) -> Option<TokenStream2> {
        let name_opt = self.ty.view_assign_tokens();
        if let Some(name) = name_opt {
            let span = self.ty.span();
            Some(quote_spanned! {
                span => #name
            })
        } else {
            None
        }
    }
}*/

#[derive(Debug)]
pub(super) struct Properties {
    pub properties: Vec<Property>,
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        let props: Punctuated<Property, Token![,]> = input.parse_terminated(Property::parse)?;
        let properties = props.into_pairs().map(|pair| pair.into_value()).collect();
        Ok(Properties { properties })
    }
}

#[derive(Debug)]
pub(super) struct WidgetFunc {
    pub path_segments: Vec<Ident>,
    pub args: Option<Punctuated<Expr, token::Comma>>,
    pub ty: Option<Vec<Ident>>,
}

impl Spanned for WidgetFunc {
    fn span(&self) -> Span2 {
        self.path_segments
            .first()
            .expect("No segments in WidgetFunc")
            .span()
    }
}

impl Parse for WidgetFunc {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut path_segments = Vec::new();
        let mut args = None;
        let mut ty = None;

        path_segments.push(input.parse()?);

        loop {
            if input.peek(Ident) {
                path_segments.push(input.parse()?);
            } else if input.peek(Token! [::]) {
                let _colon: Token![::] = input.parse()?;
            } else if input.peek(token::Paren) {
                let paren_input;
                parenthesized!(paren_input in input);
                args = Some(paren_input.call(Punctuated::parse_terminated)?);
                if input.peek(Token! [->]) {
                    let _token: Token! [->] = input.parse()?;
                    let mut ty_path = vec![input.parse()?];

                    loop {
                        if input.peek(Ident) {
                            ty_path.push(input.parse()?);
                        } else if input.peek(Token! [::]) {
                            let _colon: Token![::] = input.parse()?;
                        } else {
                            break;
                        }
                    }
                    ty = Some(ty_path);
                }
                break;
            } else {
                break;
            }
        }

        Ok(WidgetFunc {
            path_segments,
            args,
            ty,
        })
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

#[derive(Debug)]
pub(super) struct Widget {
    pub name: Ident,
    pub func: WidgetFunc,
    pub properties: Properties,
    pub wrapper: Option<Ident>,
    pub assign_as_ref: bool,
}

impl<'a> Widget {
    pub fn get_widget_list(&'a self, widgets: &mut Vec<&'a Widget>) {
        widgets.push(self);

        for prop in &self.properties.properties {
            let ty = &prop.ty;
            if let PropertyType::Widget(widget) = ty {
                widget.get_widget_list(widgets);
            }
        }
    }
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_opt: Option<Ident> = None;

        if input.peek2(Token![=]) {
            name_opt = Some(input.parse()?);
            let _token: Token![=] = input.parse()?;
        };

        let inner_input: Option<ParseBuffer>;

        let wrapper = if input.peek(Ident) && input.peek2(token::Paren) {
            let ident = input.parse()?;
            let paren_input;
            parenthesized!(paren_input in input);
            inner_input = Some(paren_input);
            Some(ident)
        } else {
            inner_input = None;
            None
        };

        let func_input = if let Some(paren_input) = &inner_input {
            &paren_input
        } else {
            input
        };

        let assign_as_ref = if func_input.peek(Token![&]) {
            let _ref: Token![&] = func_input.parse()?;
            true
        } else {
            false
        };

        let func: WidgetFunc = func_input.parse()?;

        let inner;
        let _token = braced!(inner in input);
        let properties = inner.parse()?;

        let name = if let Some(name) = name_opt {
            name
        } else {
            util::idents_to_snake_case(&func.path_segments)
        };

        Ok(Widget {
            name,
            func,
            properties,
            wrapper,
            assign_as_ref,
        })
    }
}

impl Widget {
    pub fn new(span: Span, items: &[ImplItem]) -> Result<Self> {
        let mut widgets = None;
        for item in items {
            if let ImplItem::Macro(mac) = item {
                if mac
                    .mac
                    .path
                    .segments
                    .first()
                    .expect("No path segments in macro path")
                    .ident
                    == "view"
                {
                    let tokens = mac.mac.tokens.clone();
                    let new_widgets = syn::parse_macro_input::parse::<Widget>(tokens.into())?;
                    widgets = Some(new_widgets);
                } else {
                    return Err(Error::new(item.span().unwrap().into(), "Unexpected macro"));
                }
            }
        }

        widgets.ok_or_else(|| Error::new(span.into(), "No view macro in impl block"))
    }

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
