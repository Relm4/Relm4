use gtk::prelude::*;
use relm4::{
    list_view_wrapper::{ListViewWrapper, RelmListItem},
    prelude::*,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MyListItem {
    value: u8,
}

struct Widgets {
    label: gtk::Label,
    button: gtk::CheckButton,
}

impl Drop for Widgets {
    fn drop(&mut self) {
        dbg!(self.label.label());
    }
}

impl RelmListItem for MyListItem {
    type Init = u8;
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn init(value: Self::Init) -> Self {
        Self { value }
    }

    fn setup() -> (gtk::Box, Widgets) {
        let b = gtk::Box::default();

        let label = gtk::Label::new(None);
        b.append(&label);

        let button = gtk::CheckButton::new();
        b.append(&button);

        (b, Widgets {
            label,
            button,
        })
    }

    fn bind(&mut self, root: &Self::Root, widgets: &mut Self::Widgets) {
        widgets.label.set_label(&format!("Value: {} ", self.value));
        widgets.button.set_active(self.value % 2 == 0);
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
    OnlyShowEven(bool)
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

                gtk::ToggleButton {
                    set_label: "Only show even numbers",
                    connect_clicked[sender] => move |btn| {
                        sender.input(Msg::OnlyShowEven(btn.is_active()));
                    }
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
        let mut list_view_wrapper: ListViewWrapper<MyListItem> = ListViewWrapper::with_sorting();
        list_view_wrapper.add_filter(|item| item.value % 2 == 0);
        list_view_wrapper.set_filter_status(0, false);

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
            Msg::OnlyShowEven(show_only_even) => {
                self.list_view_wrapper.set_filter_status(0, show_only_even);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.list-item-wrapper");
    app.run::<App>(0);
}
