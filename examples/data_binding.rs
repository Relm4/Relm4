use gtk::prelude::*;
use relm4::{
    binding::{Binding, BoolBinding, ConnectBindingExt, IntBinding},
    prelude::*,
    RelmObjectExt,
};

struct App {
    counter: u8,
    value: BoolBinding,
    left_margin: IntBinding,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 100),
            add_binding: (&model.left_margin, "margin-start"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => Msg::Increment,
                },

                gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => Msg::Decrement,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                    set_margin_all: 5,
                },

                gtk::ToggleButton::with_binding(&model.value) { }
            }
        }
    }

    // Initialize the component.
    fn init(
        counter: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let value = BoolBinding::default();
        let left_margin = IntBinding::default();
        let model = App {
            counter,
            value,
            left_margin,
        };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        let mut value = self.value.guard();
        println!("Value: {}", *value);

        let mut margin_left = self.left_margin.guard();

        match msg {
            Msg::Increment => {
                *value = false;
                *margin_left += 1;
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                *value = true;
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.simple");
    app.run::<App>(0);
}
