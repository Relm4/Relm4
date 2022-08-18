use quote::quote;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::LitStr;

use crate::additional_fields::AdditionalFields;
use crate::menu::Menus;
use crate::widgets::ViewWidgets;

#[derive(Default)]
pub(crate) struct ComponentVisitor {
    pub view_widgets: Option<ViewWidgets>,
    pub widgets_ty: Option<syn::Type>,
    pub init: Option<syn::ImplItemMethod>,
    pub pre_view: Option<syn::ImplItemMethod>,
    pub post_view: Option<syn::ImplItemMethod>,
    pub root_name: Option<syn::Ident>,
    pub model_name: Option<syn::Ident>,
    pub additional_fields: Option<AdditionalFields>,
    pub menus: Option<Menus>,
    pub errors: Vec<syn::Error>,
}

impl VisitMut for ComponentVisitor {
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
                    Some("include") => {
                        match mac.mac.parse_body::<LitStr>() {
                            Ok(file_name) => match std::fs::read_to_string(file_name.value()) {
                                Ok(file_content) => match syn::parse_file(&file_content) {
                                    Ok(file) => {
                                        for item in file.items {
                                            if let syn::Item::Macro(syn::ItemMacro {
                                                attrs,
                                                ident: None,
                                                mac,
                                                semi_token,
                                            }) = item
                                            {
                                                let mac = syn::ImplItemMacro {
                                                    attrs,
                                                    mac,
                                                    semi_token,
                                                };
                                                self.visit_impl_item_mut(
                                                    &mut syn::ImplItem::Macro(mac),
                                                );
                                            } else {
                                                self.errors.push(syn::Error::new_spanned(
                                                    item,
                                                    "unexpected item",
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.errors.push(e);
                                    }
                                },
                                Err(io_err) => {
                                    self.errors.push(syn::Error::new_spanned(
                                        file_name,
                                        &format!("Error opening file: {}", io_err),
                                    ));
                                }
                            },
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
            syn::ImplItem::Method(func) => match &*func.sig.ident.to_string() {
                "init" => {
                    let mut init_fn_visitor = InitFnVisitor::default();
                    init_fn_visitor.visit_impl_item_method(func);

                    self.model_name = init_fn_visitor.model_name;
                    self.root_name = init_fn_visitor.root_name;
                    self.errors.append(&mut init_fn_visitor.errors);

                    let existing = self.init.replace(func.clone());
                    if existing.is_some() {
                        self.errors
                            .push(syn::Error::new_spanned(func, "duplicate init function"));
                    }
                    remove = true;
                }
                "pre_view" => {
                    let existing = self.pre_view.replace(func.clone());
                    if existing.is_some() {
                        self.errors
                            .push(syn::Error::new_spanned(func, "duplicate pre_view function"));
                    }
                    remove = true;
                }
                "post_view" => {
                    let existing = self.post_view.replace(func.clone());
                    if existing.is_some() {
                        self.errors.push(syn::Error::new_spanned(
                            func,
                            "duplicate post_view function",
                        ));
                    }
                    remove = true;
                }
                _ => (),
            },
            _ => (),
        }

        if remove {
            *item = syn::ImplItem::Verbatim(quote! {});
        }

        visit_mut::visit_impl_item_mut(self, item);
    }

    fn visit_impl_item_type_mut(&mut self, ty: &mut syn::ImplItemType) {
        if ty.ident == "Widgets" {
            self.widgets_ty = Some(ty.ty.clone());
        }
    }
}

#[derive(Default)]
struct InitFnVisitor {
    root_name: Option<syn::Ident>,
    model_name: Option<syn::Ident>,
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
                                    &path,
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
