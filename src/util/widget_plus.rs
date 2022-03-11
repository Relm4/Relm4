use gtk::prelude::{BoxExt, Cast, FixedExt, GridExt, IsA, StyleContextExt, TextViewExt, WidgetExt};
use gtk::Widget;

/// Trait that extends [`gtk::prelude::WidgetExt`].
///
/// This trait's main goal is to reduce redundant code and
/// to provide helpful methods for the widgets macro of relm4-macros.
pub trait WidgetPlus {
    /// Set margin at start, end, top and bottom all at once.
    fn set_margin_all(&self, margin: i32);

    /// Add class name if active is [`true`] and
    /// remove class name if active is [`false`]
    fn set_class_active(&self, class: &str, active: bool);

    /// Add inline CSS instructions to a widget.
    /// ```
    /// # use relm4::WidgetPlus;
    /// # gtk::init().unwrap();
    /// # let widget = gtk::Button::new();
    /// widget.inline_css("border: 1px solid red");
    /// ```
    fn inline_css(&self, style: &str);

    /// Try to remove a widget from a widget.
    ///
    /// Returns [`true`] if the removal was successful and
    /// [`false`] if nothing was done.
    fn try_remove(&self, widget: &impl IsA<Widget>) -> bool;
}

impl<W: IsA<Widget> + WidgetExt> WidgetPlus for W {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_start(margin);
        self.set_margin_end(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }

    fn set_class_active(&self, class: &str, active: bool) {
        if active {
            self.add_css_class(class);
        } else {
            self.remove_css_class(class);
        }
    }

    fn inline_css(&self, style: &str) {
        let context = self.style_context();
        let provider = gtk::CssProvider::new();

        let data = if style.ends_with(';') {
            [b"*{", style.as_bytes(), b"}"].concat()
        } else {
            [b"*{", style.as_bytes(), b";}"].concat()
        };

        provider.load_from_data(&data);
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }

    fn try_remove(&self, widget: &impl IsA<Widget>) -> bool {
        if let Some(box_) = self.as_ref().downcast_ref::<gtk::Box>() {
            box_.remove(widget);
            true
        } else if let Some(grid) = self.as_ref().downcast_ref::<gtk::Grid>() {
            grid.remove(widget);
            true
        } else if let Some(stack) = self.as_ref().downcast_ref::<gtk::Stack>() {
            stack.remove(widget);
            true
        } else if let Some(fixed) = self.as_ref().downcast_ref::<gtk::Fixed>() {
            fixed.remove(widget);
            true
        } else if let Some(text_view) = self.as_ref().downcast_ref::<gtk::TextView>() {
            text_view.remove(widget);
            true
        } else if let Some(action_bar) = self.as_ref().downcast_ref::<gtk::ActionBar>() {
            action_bar.remove(widget);
            true
        } else if let Some(flow_box) = self.as_ref().downcast_ref::<gtk::FlowBox>() {
            flow_box.remove(widget);
            true
        } else if let Some(header_bar) = self.as_ref().downcast_ref::<gtk::HeaderBar>() {
            header_bar.remove(widget);
            true
        } else if let Some(list_box) = self.as_ref().downcast_ref::<gtk::ListBox>() {
            list_box.remove(widget);
            true
        } else {
            false
        }
    }
}
