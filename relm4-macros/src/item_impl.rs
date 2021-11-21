use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Generics, ImplItemMethod, Macro, Path, Result, Token, Type, WhereClause,
};

#[derive(Debug)]
pub(super) struct ItemImpl {
    pub outer_attrs: Option<TokenStream2>,
    pub impl_generics: Generics,
    pub trait_: Path,
    pub self_ty: Box<Type>,
    pub where_clause: Option<WhereClause>,
    pub macros: Vec<Macro>,
    pub funcs: Vec<ImplItemMethod>,
    pub brace_span: Span,
}

impl Parse for ItemImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer_attrs = if !input.peek(Token![impl]) {
            let attrs = input.call(Attribute::parse_outer)?;
            Some(quote! { #(#attrs)* })
        } else {
            None
        };
        let _impl: Token![impl] = input.parse()?;

        let impl_generics = input.parse()?;
        let trait_ = input.parse()?;

        let _for: Token![for] = input.parse()?;
        let self_ty = input.parse()?;

        let where_clause = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let brace_span = input.span();
        let braced_input;
        braced!(braced_input in input);

        let mut macros = Vec::new();
        let mut funcs = Vec::new();

        while !braced_input.is_empty() {
            if braced_input.peek2(Token![!]) {
                macros.push(braced_input.parse()?);
            } else if braced_input.peek(Token![fn]) {
                funcs.push(braced_input.parse()?);
            } else {
                return Err(braced_input.error("Expeted macro or method"));
            }
        }

        Ok(ItemImpl {
            outer_attrs,
            impl_generics,
            trait_,
            self_ty,
            where_clause,
            macros,
            funcs,
            brace_span,
        })
    }
}
