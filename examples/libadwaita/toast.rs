use gtk::prelude::*;
use relm4::prelude::*;
use std::cell::Cell;

struct App {
    activated: &'static str,
    toast: Cell<Option<adw::Toast>>,
}

#[derive(Debug)]
enum Msg {
    Activate,
    Cancel,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        adw::Window {
            set_title: Some("Simple app"),
            set_default_size: (300, 200),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Toast",
                    }
                },

                #[name = "toast_overlay"]
                adw::ToastOverlay {
                    set_vexpand: true,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                        set_margin_all: 5,
                        set_valign: gtk::Align::Center,

                        gtk::Button {
                            set_hexpand: false,
                            set_vexpand: false,
                            set_label: "Activate",
                            connect_clicked => Msg::Activate,
                        },
                        gtk::Button {
                            set_hexpand: false,
                            set_vexpand: false,
                            set_label: "Cancel",
                            connect_clicked => Msg::Cancel,
                        },
                        gtk::Text {
                            #[watch]
                            set_text: model.activated,
                        },
                    },

                    #[watch]
                    add_toast?: model.toast.take(),
                }
            }

        }
    }

    // Initialize the component.
    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            activated: "Idle",
            toast: Cell::new(None),
        };
        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Activate => {
                self.activated = "Active";
                let toast = adw::Toast::new("Activated");
                toast.set_button_label(Some("Cancel"));
                toast.set_timeout(0);
                toast.connect_button_clicked(move |this| {
                    this.dismiss();
                    sender.input(Msg::Cancel);
                });
                self.toast.set(Some(toast));
            }
            Msg::Cancel => self.activated = "Idle",
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.toast");
    app.run::<App>(());
}
