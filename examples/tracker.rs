use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use rand::prelude::IteratorRandom;
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

const ICON_LIST: &[&str] = &[
    "bookmark-new-symbolic",
    "edit-copy-symbolic",
    "edit-cut-symbolic",
    "edit-find-symbolic",
    "starred-symbolic",
    "system-run-symbolic",
    "emoji-objects-symbolic",
    "emoji-nature-symbolic",
    "display-brightness-symbolic",
];

fn random_icon_name() -> &'static str {
    ICON_LIST
        .iter()
        .choose(&mut rand::thread_rng())
        .expect("Could not choose a random icon")
}

#[derive(Debug)]
enum Msg {
    UpdateFirst,
    UpdateSecond,
}

// The track proc macro allows to easily track changes to different
// fields of the model
#[tracker::track]
struct App {
    first_icon: &'static str,
    second_icon: &'static str,
    identical: bool,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            #[track(model.changed(App::identical()))]
            set_class_active: (relm4::css::IDENTICAL, model.identical),

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_all: 10,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Image {
                        set_pixel_size: 50,
                        #[track(model.changed(App::first_icon()))]
                        set_icon_name: Some(model.first_icon),
                    },

                    gtk::Button {
                        set_label: "New random image",
                        connect_clicked => Msg::UpdateFirst,
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Image {
                        set_pixel_size: 50,
                        #[track(model.changed(App::second_icon()))]
                        set_icon_name: Some(model.second_icon),
                    },

                    gtk::Button {
                        set_label: "New random image",
                        connect_clicked => Msg::UpdateSecond,
                    }
                },
            }
        }
    }

    fn update(&mut self, msg: Msg, _sender: ComponentSender<Self>) {
        // reset tracker value of the model
        self.reset();

        match msg {
            Msg::UpdateFirst => {
                self.set_first_icon(random_icon_name());
            }
            Msg::UpdateSecond => {
                self.set_second_icon(random_icon_name());
            }
        }
        self.set_identical(self.first_icon == self.second_icon);
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            first_icon: random_icon_name(),
            second_icon: random_icon_name(),
            identical: false,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.tracker");
    app.set_global_css(".identical { background: #00ad5c; }");

    app.run::<App>(());
}
