view! {
    gtk::Window {
        set_title: Some("Simple app"),
        set_default_width: 300,
        set_default_height: 100,

        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            gtk::Button {
                set_label: "Increment",
                connect_clicked[sender] => move |_| {
                    sender.input(AppMsg::Increment);
                }
            },

            gtk::Button {
                set_label: "Decrement",
                connect_clicked[sender] => move |_| {
                    sender.input(AppMsg::Decrement);
                }
            },

            gtk::Label {
                #[watch]
                set_label: &format!("Counter: {}", model.counter),
                set_margin_all: 5,
            }
        }
    }
}