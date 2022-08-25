use relm4::gtk;

fn main() {
    relm4_macros::view! {
        gtk::Stack {
            add_child = &gtk::Separator::default() { }
                -> { set_needs_attention: false },

            add_child = &gtk::Separator::default() { }
                -> page: gtk::StackPage { set_needs_attention: false },
        }
    }
}
