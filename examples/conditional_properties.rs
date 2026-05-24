use gtk::prelude::*;
use relm4::prelude::*;

struct AppModel {
    left: bool,
    homogeneous: bool,
}

#[derive(Debug)]
enum AppMsg {
    SetLeft(bool),
    SetHomogeneous(bool),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Conditional Properties Example"),
            set_default_size: (250, 125),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                // Create a conditional widget stack.
                if model.left {
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "This is\nthe left\npage!"
                    }
                } else {
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_label: "This is the right page!"
                   }
                } -> {
                    // You can set the `gtk::Stack`'s properties here.
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                    set_halign: gtk::Align::Center,
                    #[watch]
                    set_hhomogeneous: model.homogeneous,
                    #[watch]
                    set_vhomogeneous: model.homogeneous,
                },

                gtk::CheckButton {
                    set_label: Some("Show left stack page"),
                    set_active: model.left,
                    connect_toggled[sender] => move |checkbox| sender.input(AppMsg::SetLeft(checkbox.is_active())),
                },

                gtk::CheckButton {
                    set_label: Some("Homogeneous stack"),
                    set_active: model.homogeneous,
                    connect_toggled[sender] => move |checkbox| sender.input(AppMsg::SetHomogeneous(checkbox.is_active())),
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            left: true,
            homogeneous: true,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SetLeft(left) => self.left = left,
            AppMsg::SetHomogeneous(homogeneous) => self.homogeneous = homogeneous,
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.conditional_properties");
    relm4::set_global_css("stack { border: 1px black solid; }");
    app.run::<AppModel>(());
}
