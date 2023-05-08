use std::time::Duration;

use gtk::{glib, prelude::*};
use relm4::{
    binding::{Binding, U8Binding},
    prelude::*,
    typed_list_view::{RelmListItem, TypedListView},
    RelmObjectExt,
};

struct MyListItem {
    value: u8,
    binding: U8Binding,
    handle: Option<glib::JoinHandle<()>>,
}

impl PartialEq for MyListItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for MyListItem {}

impl PartialOrd for MyListItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for MyListItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl MyListItem {
    fn new(value: u8) -> Self {
        Self {
            value,
            binding: U8Binding::new(0),
            handle: None,
        }
    }
}

struct Widgets {
    label: gtk::Label,
    label2: gtk::Label,
    button: gtk::CheckButton,
}

impl RelmListItem for MyListItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
            my_box = gtk::Box {
                #[name = "label"]
                gtk::Label {
                    set_margin_end: 10,
                },

                #[name = "label2"]
                gtk::Label {
                    set_margin_end: 10,
                },

                #[name = "button"]
                gtk::CheckButton,
            }
        }

        let widgets = Widgets {
            label,
            label2,
            button,
        };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        println!("Unbind {}", self.value);
        let Widgets {
            label,
            label2,
            button,
        } = widgets;

        let future_binding = self.binding.clone();
        self.handle = Some(relm4::spawn_local(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let mut guard = future_binding.guard();
                *guard = guard.wrapping_add(1);
            }
        }));

        label.set_label(&format!("Value: {} ", self.value));
        label2.add_write_only_binding(&self.binding, "label");
        button.set_active(self.value % 2 == 0);
    }

    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        self.handle
            .take()
            .unwrap()
            .into_source_id()
            .unwrap()
            .remove();
        *self.binding.guard() = 0;
    }
}

struct App {
    counter: u8,
    list_view_wrapper: TypedListView<MyListItem, gtk::SingleSelection>,
}

#[derive(Debug)]
enum Msg {
    Append,
    Remove,
    OnlyShowEven(bool),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Async + idiomatic list view"),
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

    fn init(
        counter: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize the ListView wrapper
        let mut list_view_wrapper: TypedListView<MyListItem, gtk::SingleSelection> =
            TypedListView::with_sorting();

        // Add a filter and disable it
        list_view_wrapper.add_filter(|item| item.value % 2 == 0);
        list_view_wrapper.set_filter_status(0, false);

        let model = App {
            counter,
            list_view_wrapper,
        };

        let my_view = &model.list_view_wrapper.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Append => {
                // Add 10 items
                for _ in 0..10 {
                    self.counter = self.counter.wrapping_add(1);
                    self.list_view_wrapper.append(MyListItem::new(self.counter));
                }
            }
            Msg::Remove => {
                // Remove the second item
                self.list_view_wrapper.remove(1);
            }
            Msg::OnlyShowEven(show_only_even) => {
                // Disable or enable the first filter
                self.list_view_wrapper.set_filter_status(0, show_only_even);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.typed-list-view-async");
    app.run::<App>(0);
}
