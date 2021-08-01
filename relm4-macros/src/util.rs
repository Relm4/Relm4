use proc_macro::Span;
use syn::{
    spanned::Spanned,
    token::{Bang, For},
    Error, Ident, Path, Result,
};

pub(super) fn trait_to_path(
    type_span: Span,
    trait_: Option<(Option<Bang>, Path, For)>,
) -> Result<Path> {
    if let Some(trt) = trait_ {
        if trt
            .1
            .segments
            .iter()
            .map(|segment| &segment.ident)
            .filter(|ident| *ident == "RelmWidgets")
            .count()
            == 1
        {
            Ok(trt.1)
        } else {
            Err(Error::new(
                trt.1.span().unwrap().into(),
                "Unknown trait. Expected RelmWidgets",
            ))
        }
    } else {
        Err(Error::new(
            type_span.into(),
            "No trait specified. Expected RelmWidgets.",
        ))
    }
}

pub(super) fn idents_to_snake_case(idents: &[Ident]) -> Ident {
    use std::sync::atomic::{AtomicU16, Ordering};
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    let val = COUNTER.fetch_add(1, Ordering::Relaxed);
    let index_str = val.to_string();

    let segements: Vec<String> = idents
        .iter()
        .map(|ident| ident.to_string().to_lowercase())
        .collect();
    let length: usize =
        segements.iter().map(|seg| seg.len() + 1).sum::<usize>() + index_str.len() + 1;
    let mut name: String = String::with_capacity(length);

    for seg in &segements {
        name.push('_');
        name.push_str(seg);
    }
    name.push('_');
    name.push_str(&index_str);

    Ident::new(&name, Span::call_site().into())
}

// TODO remove this
/*pub(super) fn _expr_to_type_path(expr: &Expr) -> Result<(Option<Path>, Ident)> {
    if let Expr::Path(expr_path) = expr.clone() {
        match expr_path.path.segments.len() {
            0 => Err(Error::new(expr.span().unwrap().into(), "TODO")),
            1 => Ok((None, expr_path.path.get_ident().unwrap().clone())),
            _ => {
                let mut path = expr_path.path;
                path.segments.pop();
                let last_path_seg = path.segments.pop().unwrap().into_value();
                path.leading_colon = None;
                Ok((Some(path), last_path_seg.ident))
            }
        }
    } else {
        Err(Error::new(expr.span().unwrap().into(), "TODO"))
    }
}*/
