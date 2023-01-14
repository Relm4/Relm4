use crate::widgets::{AssignPropertyAttr, Properties, PropertyType, Widget, WidgetTemplateAttr};

impl Widget {
    /// Don't generate any fields if the widget wasn't named by the user and
    /// isn't used for any property updates either.
    pub(crate) fn has_struct_field(&self) -> bool {
        match self.template_attr {
            WidgetTemplateAttr::None => {
                self.name_assigned_by_user || self.properties.are_properties_updated()
            }
            WidgetTemplateAttr::Template => true,
            WidgetTemplateAttr::TemplateChild => false,
        }
    }
}

impl Properties {
    pub(crate) fn are_properties_updated(&self) -> bool {
        // Is there any property with watch or track attribute?
        self.properties.iter().any(|prop| match &prop.ty {
            PropertyType::Assign(assign_prop) => matches!(
                &assign_prop.attr,
                AssignPropertyAttr::Track { .. } | AssignPropertyAttr::Watch
            ),
            _ => false,
        })
    }
}
