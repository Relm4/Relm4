use quote::quote;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

use crate::additional_fields::AdditionalFields;
use crate::menu::Menus;
use crate::widgets::ViewWidgets;

pub(crate) struct ComponentVisitor<'errors> {
    pub view_widgets: Option<ViewWidgets>,
    pub widgets_ty: Option<syn::Type>,
    pub init: Option<syn::ImplItemMethod>,
    pub root_name: Option<syn::Ident>,
    pub model_name: Option<syn::Ident>,
    pub sender_name: Option<syn::Ident>,
    pub additional_fields: Option<AdditionalFields>,
    pub menus: Option<Menus>,
    pub errors: &'errors mut Vec<syn::Error>,
}

impl<'errors> ComponentVisitor<'errors> {
    pub fn new(errors: &'errors mut Vec<syn::Error>) -> Self {
        ComponentVisitor {
            view_widgets: None,
            widgets_ty: None,
            init: None,
            root_name: None,
            model_name: None,
            sender_name: None,
            additional_fields: None,
            menus: None,
            errors,
        }
    }
}

impl VisitMut for ComponentVisitor<'_> {
    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        let mut remove = false;

        match item {
            syn::ImplItem::Macro(mac) => {
                match mac.mac.path.get_ident().map(ToString::to_string).as_deref() {
                    Some("view") => {
                        match mac.mac.parse_body::<ViewWidgets>() {
                            Ok(widgets) => {
                                let existing = self.view_widgets.replace(widgets);

                                if existing.is_some() {
                                    self.errors
                                        .push(syn::Error::new_spanned(mac, "duplicate view macro"));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };

                        remove = true;
                    }
                    Some("additional_fields") => {
                        match mac.mac.parse_body::<AdditionalFields>() {
                            Ok(fields) => {
                                let existing = self.additional_fields.replace(fields);

                                if existing.is_some() {
                                    self.errors.push(syn::Error::new_spanned(
                                        mac,
                                        "duplicate additional_fields macro",
                                    ));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };

                        remove = true;
                    }
                    Some("menu") => {
                        match mac.mac.parse_body::<Menus>() {
                            Ok(menu) => {
                                let existing = self.menus.replace(menu);

                                if existing.is_some() {
                                    self.errors
                                        .push(syn::Error::new_spanned(mac, "duplicate menu macro"));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };
                        remove = true;
                    }
                    _ => (),
                }
            }
            syn::ImplItem::Method(func) => {
                if &*func.sig.ident.to_string() == "init" {
                    let mut init_fn_visitor = InitFnVisitor::default();
                    init_fn_visitor.visit_impl_item_method(func);

                    self.model_name = init_fn_visitor.model_name;
                    self.sender_name = init_fn_visitor.sender_name;
                    self.root_name = init_fn_visitor.root_name;
                    self.errors.append(&mut init_fn_visitor.errors);

                    let existing = self.init.replace(func.clone());
                    if existing.is_some() {
                        self.errors
                            .push(syn::Error::new_spanned(func, "duplicate init function"));
                    }
                    remove = true;
                }
            }
            _ => (),
        }

        if remove {
            *item = null_item();
        }

        visit_mut::visit_impl_item_mut(self, item);
    }

    fn visit_impl_item_type_mut(&mut self, ty: &mut syn::ImplItemType) {
        if ty.ident == "Widgets" {
            self.widgets_ty = Some(ty.ty.clone());
        }
    }
}

pub(crate) struct FactoryComponentVisitor<'errors> {
    pub view_widgets: Option<ViewWidgets>,
    pub widgets_ty: Option<syn::Type>,
    pub init_widgets: Option<syn::ImplItemMethod>,
    pub root_name: Option<syn::Ident>,
    pub additional_fields: Option<AdditionalFields>,
    pub menus: Option<Menus>,
    pub errors: &'errors mut Vec<syn::Error>,
}

impl<'errors> FactoryComponentVisitor<'errors> {
    pub fn new(errors: &'errors mut Vec<syn::Error>) -> Self {
        FactoryComponentVisitor {
            view_widgets: None,
            widgets_ty: None,
            init_widgets: None,
            root_name: None,
            additional_fields: None,
            menus: None,
            errors,
        }
    }
}

impl VisitMut for FactoryComponentVisitor<'_> {
    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        let mut remove = false;

        match item {
            syn::ImplItem::Macro(mac) => {
                match mac.mac.path.get_ident().map(ToString::to_string).as_deref() {
                    Some("view") => {
                        match mac.mac.parse_body::<ViewWidgets>() {
                            Ok(widgets) => {
                                let existing = self.view_widgets.replace(widgets);

                                if existing.is_some() {
                                    self.errors
                                        .push(syn::Error::new_spanned(mac, "duplicate view macro"));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };

                        remove = true;
                    }
                    Some("additional_fields") => {
                        match mac.mac.parse_body::<AdditionalFields>() {
                            Ok(fields) => {
                                let existing = self.additional_fields.replace(fields);

                                if existing.is_some() {
                                    self.errors.push(syn::Error::new_spanned(
                                        mac,
                                        "duplicate additional_fields macro",
                                    ));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };

                        remove = true;
                    }
                    Some("menu") => {
                        match mac.mac.parse_body::<Menus>() {
                            Ok(menu) => {
                                let existing = self.menus.replace(menu);

                                if existing.is_some() {
                                    self.errors
                                        .push(syn::Error::new_spanned(mac, "duplicate menu macro"));
                                }
                            }
                            Err(e) => {
                                self.errors.push(e);
                            }
                        };
                        remove = true;
                    }
                    _ => (),
                }
            }
            syn::ImplItem::Method(func) => {
                if &*func.sig.ident.to_string() == "init_widgets" {
                    let mut init_fn_visitor = InitWidgetsFnVisitor::default();
                    init_fn_visitor.visit_impl_item_method(func);

                    self.root_name = init_fn_visitor.root_name;
                    self.errors.append(&mut init_fn_visitor.errors);

                    let existing = self.init_widgets.replace(func.clone());
                    if existing.is_some() {
                        self.errors.push(syn::Error::new_spanned(
                            func,
                            "duplicate init_widgets function",
                        ));
                    }
                    remove = true;
                }
            }
            _ => (),
        }

        if remove {
            *item = null_item();
        }

        visit_mut::visit_impl_item_mut(self, item);
    }

    fn visit_impl_item_type_mut(&mut self, ty: &mut syn::ImplItemType) {
        if ty.ident == "Widgets" {
            self.widgets_ty = Some(ty.ty.clone());
        } else if ty.ident == "Root" {
            self.errors.push(syn::Error::new_spanned(
                ty,
                "`Root` type is defined by `view!` macro",
            ));
        }
    }
}

#[derive(Debug, Default)]
struct InitFnVisitor {
    root_name: Option<syn::Ident>,
    model_name: Option<syn::Ident>,
    sender_name: Option<syn::Ident>,
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for InitFnVisitor {
    fn visit_impl_item_method(&mut self, func: &'ast syn::ImplItemMethod) {
        let root_name = match func.sig.inputs.iter().nth(1) {
            Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err(syn::Error::new_spanned(
                    pat_type,
                    "unable to determine root name",
                )),
            },
            Some(arg) => Err(syn::Error::new_spanned(
                arg,
                "unable to determine root name",
            )),
            None => Err(syn::Error::new_spanned(
                &func.sig,
                "unable to determine root name",
            )),
        };

        match root_name {
            Ok(root_name) => self.root_name = Some(root_name),
            Err(e) => self.errors.push(e),
        }

        let sender_name = match func.sig.inputs.iter().nth(2) {
            Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err(syn::Error::new_spanned(
                    pat_type,
                    "unable to determine sender name",
                )),
            },
            Some(arg) => Err(syn::Error::new_spanned(
                arg,
                "unable to determine sender name",
            )),
            None => Err(syn::Error::new_spanned(
                &func.sig,
                "unable to determine sender name",
            )),
        };

        match sender_name {
            Ok(sender_name) => self.sender_name = Some(sender_name),
            Err(e) => self.errors.push(e),
        }

        visit::visit_impl_item_method(self, func);
    }

    fn visit_expr_struct(&mut self, expr_struct: &'ast syn::ExprStruct) {
        if expr_struct.path.segments.last().unwrap().ident == "ComponentParts" {
            for field in &expr_struct.fields {
                let member_name = match &field.member {
                    syn::Member::Named(ident) => Some(ident.to_string()),
                    syn::Member::Unnamed(_) => None,
                };

                if member_name.as_deref() == Some("model") {
                    let model_name = match &field.expr {
                        syn::Expr::Path(path) => {
                            if let Some(ident) = path.path.get_ident() {
                                Ok(ident.clone())
                            } else {
                                Err(syn::Error::new_spanned(
                                    path,
                                    "unable to determine model name",
                                ))
                            }
                        }
                        _ => Err(syn::Error::new_spanned(
                            &field.expr,
                            "unable to determine model name",
                        )),
                    };

                    match model_name {
                        Ok(model_name) => self.model_name = Some(model_name),
                        Err(e) => self.errors.push(e),
                    }
                }
            }
        }

        visit::visit_expr_struct(self, expr_struct);
    }
}

#[derive(Default)]
pub struct InitWidgetsFnVisitor {
    root_name: Option<syn::Ident>,
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for InitWidgetsFnVisitor {
    fn visit_impl_item_method(&mut self, func: &'ast syn::ImplItemMethod) {
        let root_name = match func.sig.inputs.iter().nth(2) {
            Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err(syn::Error::new_spanned(
                    pat_type,
                    "unable to determine root name",
                )),
            },
            Some(arg) => Err(syn::Error::new_spanned(
                arg,
                "unable to determine root name",
            )),
            None => Err(syn::Error::new_spanned(
                &func.sig,
                "unable to determine root name",
            )),
        };

        match root_name {
            Ok(root_name) => self.root_name = Some(root_name),
            Err(e) => self.errors.push(e),
        }

        visit::visit_impl_item_method(self, func);
    }
}

pub struct PreAndPostView<'errors> {
    pub pre_view: Vec<syn::Stmt>,
    pub post_view: Vec<syn::Stmt>,
    errors: &'errors mut Vec<syn::Error>,
}

impl<'errors> PreAndPostView<'errors> {
    pub fn extract(impl_: &mut syn::ItemImpl, errors: &'errors mut Vec<syn::Error>) -> Self {
        let mut visitor = PreAndPostView {
            pre_view: vec![],
            post_view: vec![],
            errors,
        };

        visitor.visit_item_impl_mut(impl_);

        visitor
    }
}

impl VisitMut for PreAndPostView<'_> {
    fn visit_impl_item_mut(&mut self, item: &mut syn::ImplItem) {
        if let syn::ImplItem::Method(func) = item {
            match &*func.sig.ident.to_string() {
                "pre_view" => {
                    if !self.pre_view.is_empty() {
                        self.errors.push(syn::Error::new_spanned(
                            &func,
                            "duplicate pre_view function",
                        ));
                    }

                    self.pre_view = func.block.stmts.clone();
                    *item = null_item();
                }
                "post_view" => {
                    if !self.post_view.is_empty() {
                        self.errors.push(syn::Error::new_spanned(
                            &func,
                            "duplicate post_view function",
                        ));
                    }

                    self.post_view = func.block.stmts.clone();
                    *item = null_item();
                }
                _ => (),
            }
        }

        visit_mut::visit_impl_item_mut(self, item)
    }
}

/// Returns an empty impl item that can be used to remove an existing item in a mutable visitor.
fn null_item() -> syn::ImplItem {
    syn::ImplItem::Verbatim(quote! {})
}
