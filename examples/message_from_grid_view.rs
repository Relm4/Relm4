use gtk::prelude::*;
use rand::seq::IteratorRandom;
use relm4::gtk::{glib, glib::SignalHandlerId};
use relm4::{
    prelude::*,
    typed_view::grid::{RelmGridItem, TypedGridView},
};

const CONTRIBUTORS: &[&str] = &[
    "AaronErhardt",
    "MaksymShcherbak",
    "mmstick",
    "tronta",
    "zekefast",
    "M23SNEZ3",
];

fn random_name() -> &'static str {
    CONTRIBUTORS
        .iter()
        .choose(&mut rand::rng())
        .expect("Could not choose a random name")
}

#[derive(Debug, Clone)]
struct MyGridItem {
    name: &'static str,
    sender: ComponentSender<App>,
}

impl MyGridItem {
    fn new(sender: ComponentSender<App>) -> Self {
        Self {
            name: random_name(),
            sender,
        }
    }
}

struct Widgets {
    button: gtk::Button,
    label: gtk::Label,
    click_handler: Option<SignalHandlerId>,
}

impl RelmGridItem for MyGridItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 2,
                set_spacing: 5,

                #[name = "button"]
                gtk::Button {
                    set_hexpand: true,
                    #[name = "label"]
                    gtk::Label {
                        set_halign: gtk::Align::Center,
                    }
                }
            }
        }

        let widgets = Widgets {
            label,
            button,
            click_handler: None,
        };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.label.set_label(self.name);
        if let Some(id) = widgets.click_handler.take() {
            widgets.button.disconnect(id);
        }
        let name = self.name;
        let handler_id = widgets.button.connect_clicked(glib::clone!(
            #[strong(rename_to = sender)]
            self.sender,
            move |_btn| {
                // Use the cloned sender to send a message
                sender.input(Msg::Print(name));
            }
        ));
        widgets.click_handler = Some(handler_id);
    }
}

#[derive(Debug)]
struct App {
    grid_view: TypedGridView<MyGridItem, gtk::NoSelection>,
}

#[derive(Debug)]
enum Msg<'a> {
    Add,
    Print(&'a str),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = u8;
    type Input = Msg<'static>;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Is it really possible to send messages from an item in a grid?"),
            set_default_size: (650, 300),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Append 10 items",
                    connect_clicked => Msg::Add,
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,

                    #[local_ref]
                    my_view -> gtk::GridView {
                        set_orientation: gtk::Orientation::Vertical,
                        set_max_columns: 3,
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize the GridView
        let grid_view: TypedGridView<MyGridItem, gtk::NoSelection> = TypedGridView::new();

        let model = App { grid_view };

        let my_view = &model.grid_view.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Add => {
                for _ in 0..10 {
                    self.grid_view.append(MyGridItem::new(sender.clone()));
                }
            }
            Msg::Print(name) => {
                println!("Name: {:?}", name)
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.message_from_grid_view");
    app.run::<App>(0);
}
