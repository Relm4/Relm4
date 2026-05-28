use crate::widgets::{TopLevelInner, ViewWidgets};

impl ViewWidgets {
    /// Get a mutable reference to the root widget
    pub(crate) fn mark_root_as_used(&mut self) {
        if let Some(root_widget) = self
            .top_level_widgets
            .iter_mut()
            .find(|w| w.root_attr.is_some())
            && let TopLevelInner::Widget(widget) = &mut root_widget.inner
        {
            widget.name_assigned_by_user = true;
        }
    }
}
