use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

use crate::widgets::{PropertyType, ViewWidgets, Widget, WidgetTemplateAttr};

use super::WidgetFieldsScope;

impl ViewWidgets {
    /// Get a mutable reference to the root widget
    pub fn mark_root_as_used(&mut self) {
        if let Some(root_widget) = self
            .top_level_widgets
            .iter_mut()
            .find(|w| w.root_attr.is_some())
        {
            root_widget.inner.name_assigned_by_user = true;
        }
    }
}

impl Widget {
    // Find all template children and get their variables in scope.
    pub(crate) fn get_template_child_in_scope(
        &self,
        stream: &mut TokenStream2,
        scope: WidgetFieldsScope,
    ) {
        if self.template_attr == WidgetTemplateAttr::Template {
            for prop in &self.properties.properties {
                if let PropertyType::Widget(widget) = &prop.ty {
                    // Only get the child into scope during init or when its properties are updated.
                    if (scope == WidgetFieldsScope::Init
                        || widget.properties.are_properties_updated())
                        && widget.template_attr == WidgetTemplateAttr::TemplateChild
                    {
                        let template_name = &self.name;
                        let child_name = &widget.name;
                        stream.extend(quote_spanned! {
                            child_name.span() => let #child_name = &#template_name.#child_name;
                        });
                    }
                }
            }
        }
    }
}
