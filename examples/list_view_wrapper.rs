use gtk::prelude::*;
use relm4::{
    binding::U8Binding,
    list_item_wrapper::{ListViewWrapper, RelmListItem},
    prelude::*,
    RelmObjectExt,
};

struct MyListItem {
    value: u8,
}

impl RelmListItem for MyListItem {
    type Init = u8;
    type Widget = gtk::Label;

    fn init(value: Self::Init) -> Self {
        Self { value }
    }

    fn setup() -> gtk::Label {
        gtk::Label::new(None)
    }

    fn bind(&self, widget: Self::Widget) {
        widget.set_label(&format!("Value: {}", self.value));
    }

    fn unbind(&self, widget: Self::Widget) {}

    fn teardown(&self, widget: Self::Widget) {}
}
struct App {
    counter: u8,
    list_view_wrapper: ListViewWrapper<MyListItem>,
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

                gtk::ScrolledWindow {
                    #[local_ref]
                    my_view -> gtk::ListView {}
                }
            }
        }
    }

    // Initialize the component.
    fn init(
        counter: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_view_wrapper: ListViewWrapper<MyListItem> = ListViewWrapper::new();
        list_view_wrapper.append(1);
        list_view_wrapper.append(2);
        list_view_wrapper.append(3);

        let model = App {
            counter,
            list_view_wrapper,
        };

        let my_view = model.list_view_wrapper.view();

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.list-item-wrapper");
    app.run::<App>(0);
}
