use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{Ident, Path};

use super::{util, Property, PropertyType};

impl PropertyType {
    fn connect_parent_tokens(&self) -> Option<TokenStream2> {
        if let PropertyType::Parent(expr) = self {
            //Some(parent_tokens(expr))
            Some(expr.to_token_stream())
        } else {
            None
        }
    }
}

impl Property {
    pub fn connect_parent_stream(
        &self,
        stream: &mut TokenStream2,
        parent_name: &Ident,
        relm4_path: &Path,
    ) {
        if let Some(p_assign) = self.ty.connect_parent_tokens() {
            let args_stream = self.args_stream();

            // Parents are only for the widget macro, therefore self is never the widgets
            let assign_fn =
                self.name
                    .self_assign_fn_stream(&self.generics, parent_name, false, relm4_path);
            let self_assign_args = self.name.assign_args_stream(parent_name);

            util::property_assign_tokens(
                stream,
                self,
                assign_fn,
                self_assign_args,
                p_assign,
                None,
                args_stream,
            );
        }
    }
}

// fn parent_ident(path: &ExprPath) -> TokenStream2 {
//     if path.path.segments.len() == 1 {
//         let ident = &path.path.segments.first().unwrap().ident;
//         quote_spanned! { path.span() => parent_widgets.#ident.root_widget() }
//     } else {
//         path.to_token_stream()
//     }
// }

// fn parent_tokens(expr: &Expr) -> TokenStream2 {
//     match expr {
// Expr::Call(call) => {
//     if let Expr::Path(path) = &*call.func {
//         if let Some(segs) = path.path.segments.first() {
//             if segs.ident == "Some" {
//                 if call.args.len() == 1 {
//                     if let Expr::Path(args_path) = call.args.first().unwrap() {
//                         let arg_tokens = parent_ident(args_path);
//                         quote_spanned! { path.span() => Some(#arg_tokens) }
//                     } else {
//                         expr.to_token_stream()
//                     }
//                 } else {
//                     expr.to_token_stream()
//                 }
//             } else {
//                 expr.to_token_stream()
//             }
//         } else {
//             expr.to_token_stream()
//         }
//     } else {
//         expr.to_token_stream()
//     }
// }
//         Expr::Path(path) => parent_ident(path),
//         _ => expr.to_token_stream(),
//     }
// }
