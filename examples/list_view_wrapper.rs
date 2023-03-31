use gtk::prelude::*;
use relm4::{
    list_item_wrapper::{ListViewWrapper, RelmListItem},
    prelude::*,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
}

struct App {
    counter: u8,
    list_view_wrapper: ListViewWrapper<MyListItem>,
}

#[derive(Debug)]
enum Msg {
    Append,
    Remove,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Actually idiomatic list view possible?"),
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Append 10 items",
                    connect_clicked => Msg::Append,
                },

                gtk::Button {
                    set_label: "Remove second item",
                    connect_clicked => Msg::Remove,
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,

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
        let list_view_wrapper: ListViewWrapper<MyListItem> = ListViewWrapper::with_sorting();

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
            Msg::Append => {
                for _ in 0..10 {
                    self.counter = self.counter.wrapping_add(1);
                    self.list_view_wrapper.append(self.counter);
                }
            }
            Msg::Remove => {
                self.list_view_wrapper.remove(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.list-item-wrapper");
    app.run::<App>(0);
}
