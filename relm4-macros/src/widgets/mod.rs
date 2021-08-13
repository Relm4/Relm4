/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use proc_macro2::Span as Span2;
use proc_macro2::TokenStream as TokenStream2;
use syn::{punctuated::Punctuated, spanned::Spanned, token, Expr, ExprClosure, Ident, Lit};

use crate::args::Args;

mod gen;
mod parse;

#[derive(Debug)]
pub(super) struct Tracker {
    items: Vec<Expr>,
    update_fn: Expr,
}

#[derive(Debug)]
pub(super) enum PropertyType {
    Expr(Expr),
    Value(Lit),
    Track(Tracker),
    Component(Expr),
    Args(Args<Ident>),
    Connect(ExprClosure),
    Watch(TokenStream2),
    Factory(Expr),
    Widget(Widget),
}

#[derive(Debug)]
pub(super) struct Property {
    pub name: Ident,
    pub ty: PropertyType,
    pub args: Option<Args<Expr>>,
    pub optional_assign: bool,
    pub iterative: bool,
}

#[derive(Debug)]
pub(super) struct Properties {
    pub properties: Vec<Property>,
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
