use proc_macro::TokenStream;
use proc_macro2::{Span as Span2, TokenStream as TokenStream2};
use syn::punctuated::Punctuated;

use syn::spanned::Spanned;
use syn::{FnArg, Ident, ImplItem, ItemImpl, Path, PathArguments, PathSegment, Type, TypePath};

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
                ident: Ident::new(string, Span2::mixed_site()),
                arguments: PathArguments::None,
            }
        })
        .collect();
    Path {
        leading_colon: None,
        segments: Punctuated::from_iter(path_segments),
    }
}

pub(super) fn item_impl_error(original_input: TokenStream) -> TokenStream {
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
    vec![macro_impls, original_input].into_iter().collect()
}

pub(super) fn verbatim_impl_item_fn(
    name: &str,
    args: Vec<FnArg>,
    ty: Type,
    tokens: TokenStream2,
) -> ImplItem {
    ImplItem::Fn(syn::ImplItemFn {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: syn::Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: syn::token::Fn::default(),
            ident: Ident::new(name, Span2::mixed_site()),
            generics: syn::Generics {
                lt_token: None,
                params: Punctuated::default(),
                gt_token: None,
                where_clause: None,
            },
            paren_token: syn::token::Paren::default(),
            inputs: Punctuated::from_iter(args),
            variadic: None,
            output: syn::ReturnType::Type(syn::token::RArrow::default(), Box::new(ty)),
        },
        block: syn::Block {
            brace_token: syn::token::Brace::default(),
            stmts: vec![syn::Stmt::Expr(syn::Expr::Verbatim(tokens), None)],
        },
    })
}

/// Returns the identifier for a parameter if it is a binding with an identifier pattern.
pub(super) fn extract_arg_ident(arg: &FnArg) -> syn::Result<&Ident> {
    match arg {
        syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
            syn::Pat::Ident(ident) => Ok(&ident.ident),
            pat => Err(syn::Error::new_spanned(
                pat,
                "parameter binding must be an identifier",
            )),
        },
        syn::FnArg::Receiver(_) => Err(syn::Error::new_spanned(
            arg,
            "parameter binding must be an identifier",
        )),
    }
}
