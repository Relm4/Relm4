use gtk::prelude::*;
use relm4::{
    binding::{Binding, U8Binding},
    prelude::*,
    typed_view::{
        column::{LabelColumn, RelmColumn, TypedColumnView},
        OrdFn,
    },
    RelmObjectExt,
};

#[derive(Debug, PartialEq, Eq)]
struct MyListItem {
    value: u8,
    binding: U8Binding,
}

impl MyListItem {
    fn new(value: u8) -> Self {
        Self {
            value,
            binding: U8Binding::new(0),
        }
    }
}

struct Label1Column;

impl LabelColumn for Label1Column {
    type Item = MyListItem;
    type Value = u8;

    const COLUMN_NAME: &'static str = "label";

    const ENABLE_SORT: bool = true;
    const ENABLE_RESIZE: bool = true;

    fn get_cell_value(item: &Self::Item) -> Self::Value {
        item.value
    }

    fn format_cell_value(value: &Self::Value) -> String {
        format!("Value: {} ", value)
    }
}

struct Label2Column;

impl RelmColumn for Label2Column {
    type Root = gtk::Label;
    type Widgets = ();
    type Item = MyListItem;

    const COLUMN_NAME: &'static str = "label2";

    fn setup(_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        (gtk::Label::new(None), ())
    }

    fn bind(item: &mut Self::Item, _: &mut Self::Widgets, label: &mut Self::Root) {
        label.add_write_only_binding(&item.binding, "label");
    }

    fn sort_fn() -> OrdFn<Self::Item> {
        Some(Box::new(|a, b| a.value.cmp(&b.value)))
    }
}

struct ButtonColumn;

impl RelmColumn for ButtonColumn {
    type Root = gtk::CheckButton;
    type Widgets = ();
    type Item = MyListItem;

    const COLUMN_NAME: &'static str = "button";
    const ENABLE_EXPAND: bool = true;

    fn setup(_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        (gtk::CheckButton::new(), ())
    }

    fn bind(item: &mut Self::Item, _: &mut Self::Widgets, button: &mut Self::Root) {
        button.set_active(item.value % 2 == 0);
    }
}

struct App {
    counter: u8,
    view_wrapper: TypedColumnView<MyListItem, gtk::SingleSelection>,
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
            set_title: Some("Actually idiomatic view possible?"),
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
                    my_view -> gtk::ColumnView {}
                }
            }
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize the ListView wrapper
        let mut view_wrapper = TypedColumnView::<MyListItem, gtk::SingleSelection>::new();
        view_wrapper.append_column::<Label1Column>();
        view_wrapper.append_column::<Label2Column>();
        view_wrapper.append_column::<ButtonColumn>();

        // Add a filter and disable it
        view_wrapper.add_filter(|item| item.value % 2 == 0);
        view_wrapper.set_filter_status(0, false);

        let model = App {
            counter,
            view_wrapper,
        };

        let my_view = &model.view_wrapper.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Append => {
                // Add 10 items
                for _ in 0..10 {
                    self.counter = self.counter.wrapping_add(1);
                    self.view_wrapper.append(MyListItem::new(self.counter));
                }

                // Count up the first item
                let first_item = self.view_wrapper.get(0).unwrap();
                let first_binding = &mut first_item.borrow_mut().binding;
                let mut guard = first_binding.guard();
                *guard += 1;
            }
            Msg::Remove => {
                // Remove the second item
                self.view_wrapper.remove(1);
            }
            Msg::OnlyShowEven(show_only_even) => {
                // Disable or enable the first filter
                self.view_wrapper.set_filter_status(0, show_only_even);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.typed-column-view");
    app.run::<App>(0);
}
