use proc_macro2::Span as Span2;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::{And, Paren};
use syn::{Expr, ExprCall, ExprLit, ExprPath, Ident, Lit, LitStr};

use crate::args::Args;
use crate::widgets::{parse_util, IfBranch, ParseError, Widget};

impl IfBranch {
    pub(super) fn parse(input: ParseStream, index: usize) -> Result<Self, ParseError> {
        let cond = input.parse()?;

        let braced = parse_util::braces(input)?;

        let attributes = braced.parse().ok();
        let args = args_from_index(index);
        let mut widget = Widget::parse(&braced, attributes, Some(args))?;
        widget.ref_token = Some(And {
            spans: [Span2::call_site()],
        });

        Ok(Self { cond, widget })
    }
}

pub(super) fn args_from_index(index: usize) -> Args<Expr> {
    Args {
        inner: vec![Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Ident::new("Some", Span2::call_site()).into(),
            })),
            paren_token: Paren {
                span: Span2::call_site(),
            },
            args: Punctuated::from_iter(vec![Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Str(LitStr::new(&format!("{index}"), Span2::call_site())),
            })]),
        })],
    }
}
