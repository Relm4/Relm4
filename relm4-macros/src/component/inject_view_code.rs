use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, Expr, ExprCall, Ident, ImplItemMethod, Pat, Result, Stmt};

pub(crate) fn inject_view_code(
    mut func: ImplItemMethod,
    view_code: TokenStream2,
    widgets_return_code: TokenStream2,
) -> Result<ImplItemMethod> {
    let func_span = func.span();
    let mut stmts = func.block.stmts;

    if stmts.is_empty() {
        return Err(Error::new(
            func_span,
            "The `init` function must not be empty",
        ));
    }

    let mut new_stmts = Vec::new();
    let mut iter = stmts.drain(..);

    let view_code_stmt = loop {
        if let Some(stmt) = iter.next() {
            let mut widget_ident = None;
            if let Stmt::Local(local) = &stmt {
                let pat = &local.pat;
                if let Some(init) = &local.init {
                    if let Expr::Call(call) = &*init.1 {
                        if let Some(ident) = fn_name_from_call_expr(call) {
                            if ident == "view_output" {
                                if let Pat::Ident(ident) = &pat {
                                    widget_ident = Some(ident);
                                } else {
                                    return Err(Error::new(pat.span(), "Expected an identifier"));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ident) = widget_ident {
                break Stmt::Expr(Expr::Verbatim(quote! {
                    #view_code
                    let #ident = #widgets_return_code;
                }));
            } else {
                new_stmts.push(stmt);
            }
        } else {
            return Err(Error::new(func_span, "Expected an injection point for the view macro. Try using `let widgets = view_output();`"));
        }
    };
    new_stmts.push(view_code_stmt);

    // Push the remaining statements.
    for stmt in iter {
        new_stmts.push(stmt);
    }

    func.block.stmts = new_stmts;
    Ok(func)
}

fn fn_name_from_call_expr(expr_call: &ExprCall) -> Option<Ident> {
    if let Expr::Path(func_path) = &*expr_call.func {
        if let Some(ident) = func_path.path.get_ident() {
            if expr_call.args.is_empty() {
                Some(ident.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
