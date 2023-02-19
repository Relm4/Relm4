use gtk::{
    prelude::{Cast, StaticType, WidgetExt},
    traits::StyleContextExt,
};

/// Trait that extends [`gtk::prelude::WidgetExt`].
///
/// This trait's main goal is to reduce redundant code and
/// to provide helpful methods for the widgets macro of relm4-macros.
pub trait RelmWidgetExt {
    /// Attach widget to a `gtk::SizeGroup`.
    fn set_size_group(&self, size_group: &gtk::SizeGroup);

    /// Locate the top level window this widget is attached to.
    ///
    /// Equivalent to `widget.ancestor(gtk::Window::static_type())`, then casting.
    fn toplevel_window(&self) -> Option<gtk::Window>;

    /// Set margin at start, end, top and bottom all at once.
    fn set_margin_all(&self, margin: i32);

    /// Add class name if active is [`true`] and
    /// remove class name if active is [`false`]
    fn set_class_active(&self, class: &str, active: bool);

    /// Add inline CSS instructions to a widget.
    /// ```
    /// # use relm4::RelmWidgetExt;
    /// # gtk::init().unwrap();
    /// # let widget = gtk::Button::new();
    /// widget.inline_css("border: 1px solid red");
    /// ```
    fn inline_css(&self, style: &str);
}

impl<T: gtk::glib::IsA<gtk::Widget>> RelmWidgetExt for T {
    fn set_size_group(&self, size_group: &gtk::SizeGroup) {
        size_group.add_widget(self);
    }

    fn toplevel_window(&self) -> Option<gtk::Window> {
        self.ancestor(gtk::Window::static_type())
            .and_then(|widget| widget.dynamic_cast::<gtk::Window>().ok())
    }

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
            ["*{", style, "}"].concat()
        } else {
            ["*{", style, ";}"].concat()
        };

        provider.load_from_data(&data);
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }
}
