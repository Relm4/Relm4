use relm4::gtk;

fn widget() -> gtk::Separator {
    // Mimic component.widget() call
    gtk::Separator::default()
}

fn main() {
    let local_widget = widget();
    relm4_macros::view! {
        gtk::Stack {
            add_child = &gtk::Separator::default() { }
                -> { set_needs_attention: false },
            add_child = &gtk::Separator::default() { }
                -> page: gtk::StackPage { set_needs_attention: false },
            add_child = &widget() {} -> {
                set_needs_attention: false
            },
            add_child: &widget(),
            #[local_ref]
            add_child = &local_widget {} -> {
                set_needs_attention: false
            },
        }
    }
}
