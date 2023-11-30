use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{punctuated::Punctuated, token, Ident};

use crate::widgets::{
    AssignProperty, AssignPropertyAttr, ConditionalBranches, ConditionalWidget, MatchArm,
    Properties, Property, PropertyName, PropertyType, ReturnedWidget, Widget, WidgetTemplateAttr,
};

use super::assign::AssignInfo;

impl Property {
    fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        widget_name: &Ident,
        template_path: Option<Punctuated<Ident, token::Dot>>,
        model_name: &Ident,
        conditional_branch: bool,
    ) {
        match &self.ty {
            PropertyType::Assign(assign) => assign.update_view_stream(
                stream,
                &self.name,
                widget_name,
                template_path,
                model_name,
                conditional_branch,
            ),
            PropertyType::Widget(widget) => {
                widget.update_view_stream(
                    stream,
                    Some(widget_name),
                    model_name,
                    conditional_branch,
                );
            }
            PropertyType::ConditionalWidget(cond_widget) => {
                cond_widget.update_view_stream(stream, model_name);
            }
            PropertyType::SignalHandler(_) | PropertyType::ParseError(_) => (),
        }
    }
}

impl Properties {
    fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        widget_name: &Ident,
        template_path: Option<Punctuated<Ident, token::Dot>>,
        model_name: &Ident,
        conditional_branch: bool,
    ) {
        for prop in &self.properties {
            prop.update_view_stream(
                stream,
                widget_name,
                template_path.clone(),
                model_name,
                conditional_branch,
            );
        }
    }
}

impl Widget {
    pub(crate) fn init_update_view_stream(&self, stream: &mut TokenStream2, model_name: &Ident) {
        self.update_view_stream(stream, None, model_name, false);
    }

    fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        parent_widget_name: Option<&Ident>,
        model_name: &Ident,
        conditional_branch: bool,
    ) {
        let widget_name = &self.name;
        let template_path = if self.template_attr == WidgetTemplateAttr::TemplateChild {
            parent_widget_name.map(|parent_widget_name| {
                self.func
                    .widget_template_path(parent_widget_name, &self.name)
            })
        } else {
            None
        };

        self.properties.update_view_stream(
            stream,
            widget_name,
            template_path,
            model_name,
            conditional_branch,
        );
        if let Some(returned_widget) = &self.returned_widget {
            returned_widget.update_view_stream(stream, model_name, conditional_branch);
        }
    }
}

impl ConditionalWidget {
    fn update_view_stream(&self, stream: &mut TokenStream2, model_name: &Ident) {
        let brach_stream = match &self.branches {
            ConditionalBranches::If(if_branches) => {
                let mut stream = TokenStream2::new();

                for (index, branch) in if_branches.iter().enumerate() {
                    let mut inner_update_stream = TokenStream2::new();
                    branch.widget.update_view_stream(
                        &mut inner_update_stream,
                        None,
                        model_name,
                        true,
                    );
                    branch.update_stream(&mut stream, &inner_update_stream, index);
                }
                stream
            }
            ConditionalBranches::Match((match_token, expr, match_arms)) => {
                let mut inner_tokens = TokenStream2::new();
                for (index, match_arm) in match_arms.iter().enumerate() {
                    let mut inner_update_stream = TokenStream2::new();
                    match_arm.widget.update_view_stream(
                        &mut inner_update_stream,
                        None,
                        model_name,
                        true,
                    );
                    let MatchArm {
                        pattern,
                        guard,
                        arrow,
                        ..
                    } = match_arm;
                    let (guard_if, guard_expr) = if let Some((guard_if, guard_expr)) = guard {
                        (Some(guard_if), Some(guard_expr))
                    } else {
                        (None, None)
                    };

                    let index = index.to_string();
                    inner_tokens.extend(quote! {
                        #pattern #guard_if #guard_expr #arrow {
                            let page_active: bool = (current_page == #index);
                            #inner_update_stream
                            #index
                        },
                    });
                }
                quote! {
                    #match_token #expr {
                        #inner_tokens
                    }
                }
            }
        };

        let w_name = &self.name;
        stream.extend(quote! {
            let current_page = #w_name.visible_child_name().map_or("".to_string(), |s| s.as_str().to_string());
            #w_name.set_visible_child_name(#brach_stream);
        });
    }
}

impl ReturnedWidget {
    fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        model_name: &Ident,
        conditional_branch: bool,
    ) {
        let w_name = &self.name;
        self.properties
            .update_view_stream(stream, w_name, None, model_name, conditional_branch);
    }
}

impl AssignProperty {
    fn update_view_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        widget_name: &Ident,
        template_path: Option<Punctuated<Ident, token::Dot>>,
        model_name: &Ident,
        conditional_branch: bool,
    ) {
        match &self.attr {
            AssignPropertyAttr::None => (),
            AssignPropertyAttr::Watch { .. } => {
                let mut info = AssignInfo {
                    stream,
                    widget_name,
                    template_path,
                    is_conditional: false,
                };
                self.assign_stream(&mut info, p_name, false);
            }
            AssignPropertyAttr::Track {
                track_expr,
                paste_model,
                ..
            } => {
                let mut assign_stream = TokenStream2::new();
                let mut info = AssignInfo {
                    stream: &mut assign_stream,
                    widget_name,
                    template_path,
                    is_conditional: false,
                };
                self.assign_stream(&mut info, p_name, false);
                let model = paste_model.then(|| model_name);
                let page_switch = conditional_branch.then(|| {
                    quote! {
                        !page_active ||
                    }
                });

                stream.extend(quote! {
                    if #page_switch (#model #track_expr) {
                        #assign_stream
                    }
                });
            }
        }
    }
}
