use proc_macro2::Span;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Generics, Macro, Path, Result, Token, Type, WhereClause,
};

#[derive(Debug)]
pub(super) struct ItemImpl {
    pub impl_generics: Generics,
    pub trait_: Path,
    pub self_ty: Box<Type>,
    pub where_clause: Option<WhereClause>,
    pub macros: Vec<Macro>,
    pub brace_span: Span,
}

impl Parse for ItemImpl {
    fn parse(input: ParseStream) -> Result<Self> {
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
        while !braced_input.is_empty() {
            macros.push(braced_input.parse()?);
        }

        Ok(ItemImpl {
            impl_generics,
            trait_,
            self_ty,
            where_clause,
            macros,
            brace_span,
        })
    }
}
