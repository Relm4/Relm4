use crate::widgets::{AssignPropertyAttr, Properties, PropertyType, Widget};

impl Widget {
    /// Don't generate any fields if the widget wasn't named by the user and
    /// isn't used for any property updates either.
    pub fn has_struct_field(&self) -> bool {
        self.name_assigned_by_user || self.properties.are_properties_updated()
    }
}

impl Properties {
    fn are_properties_updated(&self) -> bool {
        // Is there any property with watch or track attribute?
        self.properties.iter().any(|prop| match &prop.ty {
            PropertyType::Assign(assign_prop) => matches!(
                &assign_prop.attr,
                AssignPropertyAttr::Track(_) | AssignPropertyAttr::Watch
            ),
            _ => false,
        })
    }
}
