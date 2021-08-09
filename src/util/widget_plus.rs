use gtk::prelude::StyleContextExt;

pub trait WidgetPlus {
    fn set_margin_all(&self, margin: i32);
    fn add_class_name(&self, class: &str);
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

    fn inline_css(&self, style_data: &[u8]) {
        let context = self.style_context();
        let provider = gtk::CssProvider::new();
        provider.load_from_data(&[b"*{", style_data, b"}"].concat());
        context.add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }
}
