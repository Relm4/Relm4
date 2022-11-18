use proc_macro::TokenStream;
use proc_macro2::{
    Delimiter as Delimiter2, Group as Group2, Punct as Punct2, Spacing as Spacing2, Span as Span2,
    TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use syn::punctuated::Punctuated;

use syn::spanned::Spanned;
use syn::{Ident, ImplItem, ItemImpl, Path, PathArguments, PathSegment, Type, TypePath};

pub(super) fn generate_widgets_type(
    widgets_ty: Option<Type>,
    component_impl: &mut ItemImpl,
    errors: &mut Vec<syn::Error>,
) -> Option<Type> {
    // Use the widget type if used.
    if let Some(ty) = widgets_ty {
        Some(ty)
    }
    // Try to generate the type from the self type name.
    else if let Type::Path(self_ty) = &*component_impl.self_ty {
        let (path, impl_item) = self_ty_to_widgets_ty(self_ty);
        component_impl.items.push(impl_item);
        Some(path)
    }
    // Error: No Widget type found or generated.
    else {
        let msg = "no `Widgets` type found and the type if `Self` in not a path. \
            Please use a path for `Self` or use `type Widgets = WidgetsName;` to name the widgets type.";
        errors.push(syn::Error::new(
            component_impl
                .items
                .first()
                .map(|i| i.span())
                .unwrap_or_else(|| component_impl.self_ty.span()),
            msg,
        ));
        None
    }
}

pub(super) fn self_ty_to_widgets_ty(self_ty: &TypePath) -> (Type, ImplItem) {
    // Retrieve path, remove any generics and append "Widgets" to the last segment.
    let mut self_path = self_ty.clone();
    let last_seg = self_path.path.segments.last_mut().unwrap();
    last_seg.arguments = Default::default();
    last_seg.ident = Ident::new(&format!("{}Widgets", last_seg.ident), last_seg.span());

    // Generate impl item for the trait impl
    let impl_item = syn::parse_quote_spanned! {
        self_path.span() => type Widgets = #self_path;
    };

    (Type::Path(self_path), impl_item)
}

pub(super) fn strings_to_path(strings: &[&str]) -> Path {
    let path_segments: Vec<PathSegment> = strings
        .iter()
        .map(|string| -> PathSegment {
            PathSegment {
                ident: Ident::new(string, Span2::call_site()),
                arguments: PathArguments::None,
            }
        })
        .collect();
    Path {
        leading_colon: None,
        segments: Punctuated::from_iter(path_segments),
    }
}

pub(super) fn item_impl_error(
    original_input: TokenStream,
    generated_types: &[&str],
    generated_fns: &[&str],
) -> TokenStream {
    // Make sure that no error occurs because of `view!` or `view_output!`
    let macro_impls = quote::quote! {
        macro_rules! view_output {
            () => { () };
        }
        macro_rules! view {
            () => {};
            ($tt:tt) => {};
            ($tt:tt $($y:tt)+) => {}
        }
    }
    .into();

    let input = add_placeholders(original_input.into(), generated_types, generated_fns);

    vec![macro_impls, input].into_iter().collect()
}

#[derive(PartialEq, Eq)]
enum ImplItemStartToken {
    None,
    Type,
    Fn,
}

fn add_placeholders(
    input: TokenStream2,
    generated_types: &[&str],
    generated_fns: &[&str],
) -> TokenStream {
    let span = input.span();
    let mut token_trees: Vec<TokenTree2> = input.into_iter().collect();
    let mut found_types = vec![false; generated_types.len()];
    let mut found_fns = vec![false; generated_fns.len()];

    let last_token_tree = token_trees.pop().unwrap();
    if let TokenTree2::Group(last_tree) = last_token_tree {
        let inner_trees: Vec<TokenTree2> = last_tree.stream().into_iter().collect();
        let mut start_token = ImplItemStartToken::None;

        for tree in &inner_trees {
            match start_token {
                ImplItemStartToken::None => {
                    if let TokenTree2::Ident(ident) = tree {
                        let ident = ident.to_string();
                        if ident == "type" {
                            start_token = ImplItemStartToken::Type;
                        } else if ident == "fn" {
                            start_token = ImplItemStartToken::Fn;
                        }
                    }
                }
                ImplItemStartToken::Type => {
                    if let TokenTree2::Ident(ident) = tree {
                        let ident = ident.to_string();
                        if let Some(pos) = generated_types.iter().position(|t| t == &ident) {
                            found_types[pos] = true;
                        }
                    }
                    start_token = ImplItemStartToken::None;
                }
                ImplItemStartToken::Fn => {
                    if let TokenTree2::Ident(ident) = tree {
                        let ident = ident.to_string();
                        if let Some(pos) = generated_fns.iter().position(|f| f == &ident) {
                            found_fns[pos] = true;
                        }
                    }
                    start_token = ImplItemStartToken::None;
                }
            }
        }

        let mut additional_trees: Vec<TokenTree2> = Vec::new();
        for (found, ty_name) in found_types.iter().zip(generated_types.iter()) {
            if !*found {
                gen_empty_ty(&mut additional_trees, ty_name, span);
            }
        }

        for (found, fn_name) in found_fns.iter().zip(generated_fns.iter()) {
            if !*found {
                gen_empty_fn(&mut additional_trees, fn_name, span);
            }
        }

        let delimiter = last_tree.delimiter();
        let stream = additional_trees
            .into_iter()
            .chain(inner_trees.into_iter())
            .collect();
        token_trees.push(TokenTree2::Group(Group2::new(delimiter, stream)));
    } else {
        token_trees.push(last_token_tree);
    };

    token_trees.into_iter().collect::<TokenStream2>().into()
}

fn gen_empty_ty(tt: &mut Vec<TokenTree2>, name: &str, span: Span2) {
    tt.push(TokenTree2::Ident(Ident::new("type", span)));
    tt.push(TokenTree2::Ident(Ident::new(name, span)));
    tt.push(TokenTree2::Punct(Punct2::new('=', Spacing2::Alone)));
    tt.push(TokenTree2::Group(Group2::new(
        Delimiter2::Parenthesis,
        TokenStream2::new(),
    )));
    tt.push(TokenTree2::Punct(Punct2::new(';', Spacing2::Alone)));
}

fn gen_empty_fn(tt: &mut Vec<TokenTree2>, name: &str, span: Span2) {
    tt.push(TokenTree2::Ident(Ident::new("fn", span)));
    tt.push(TokenTree2::Ident(Ident::new(name, span)));
    tt.push(TokenTree2::Group(Group2::new(
        Delimiter2::Parenthesis,
        TokenStream2::new(),
    )));
    tt.push(TokenTree2::Group(Group2::new(
        Delimiter2::Brace,
        TokenStream2::new(),
    )));
}
