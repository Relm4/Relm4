//! Trait that extends [`gtk::prelude::WidgetExt`].
use gtk::prelude::StyleContextExt;

/// Trait that extends [`gtk::prelude::WidgetExt`].
///
/// This trait's main goal is to reduce redundant code and
/// to provide helpful methods for the widgets macro of relm4-macros.
pub trait WidgetPlus {
    /// Set margin at start, end, top and bottom all at once.
    fn set_margin_all(&self, margin: i32);

    /// Add class name to self that can be used inside CSS selectors.
    fn add_class_name(&self, class: &str);

    /// Remove class name from self.
    fn remove_class_name(&self, class: &str);

    /// Add class name if active is [`true`] and
    /// remove class name if active is [`false`]
    fn set_class_active(&self, class: &str, active: bool) {
        if active {
            self.add_class_name(class);
        } else {
            self.remove_class_name(class);
        }
    }

    /// Add inline CSS instructions to a widget.
    /// ```
    /// # use relm4::WidgetPlus;
    /// # gtk::init().unwrap();
    /// # let widget = gtk::Button::new();
    /// widget.inline_css(b"border: 1px solid red");
    fn inline_css(&self, style_data: &[u8]);
}

impl<W: gtk::prelude::WidgetExt> WidgetPlus for W {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_start(margin);
        self.set_margin_end(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }

    fn add_class_name(&self, class: &str) {
        self.style_context().add_class(class);
    }

    fn remove_class_name(&self, class: &str) {
        self.style_context().remove_class(class);
    }

    fn inline_css(&self, style_data: &[u8]) {
        let context = self.style_context();
        let provider = gtk::CssProvider::new();
        provider.load_from_data(&[b"*{", style_data, b"}"].concat());
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }
}
