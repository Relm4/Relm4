use std::fmt::Debug;

use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::RelmRemoveAllExt;

mod css {
    pub const COMPONENT_SPACING: i32 = 5;
}

enum AppMode {
    Initial(#[allow(dead_code)] Controller<InitialScreen>),
    SubScreen1(#[allow(dead_code)] Controller<SubScreen1>),
    SubScreen2(#[allow(dead_code)] Controller<SubScreen2>),
}

struct App {
    mode: Option<AppMode>,
}

#[derive(Debug)]
enum Msg {
    ShowInitialScreen,
    ShowSubScreen1,
    ShowSubScreen2,
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_title: Some("Drop sub components"),
            set_decorated: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                gtk::HeaderBar {},
                #[name="container"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self { mode: None };
        let mut widgets = view_output!();
        model.update_with_view(&mut widgets, Msg::ShowInitialScreen, sender, &root);
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        widgets.container.remove_all();
        match message {
            Msg::ShowInitialScreen => {
                let controller =
                    InitialScreen::builder()
                        .launch(())
                        .forward(sender.input_sender(), |msg| match msg {
                            InitialScreenOutput::ShowSubScreen1 => Msg::ShowSubScreen1,
                            InitialScreenOutput::ShowSubScreen2 => Msg::ShowSubScreen2,
                        });
                widgets.container.append(controller.widget());
                self.mode = Some(AppMode::Initial(controller));
            }
            Msg::ShowSubScreen1 => {
                let controller = SubScreen1::builder()
                    .launch(())
                    .forward(sender.input_sender(), |_| Msg::ShowInitialScreen);
                widgets.container.append(controller.widget());
                self.mode = Some(AppMode::SubScreen1(controller));
            }
            Msg::ShowSubScreen2 => {
                let controller = SubScreen2::builder()
                    .launch(())
                    .forward(sender.input_sender(), |_| Msg::ShowInitialScreen);
                widgets.container.append(controller.widget());
                self.mode = Some(AppMode::SubScreen2(controller));
            }
        }
        root.set_default_size(400, 300);
    }
}

struct InitialScreen {}

#[derive(Debug)]
enum InitialScreenOutput {
    ShowSubScreen1,
    ShowSubScreen2,
}

#[relm4::component]
//noinspection RsSortImplTraitMembers
impl SimpleComponent for InitialScreen {
    type Input = ();
    type Output = InitialScreenOutput;
    type Init = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: css::COMPONENT_SPACING,
                gtk::Button {
                    set_label: "Sub screen 1",
                    connect_clicked[sender] => move |_| sender.output(InitialScreenOutput::ShowSubScreen1).unwrap()
                },
                gtk::Button {
                    set_label: "Sub screen 2",
                    connect_clicked[sender] => move |_| sender.output(InitialScreenOutput::ShowSubScreen2).unwrap()
                },
                gtk::Label {
                    set_label: "Inspect console to see what happens\nwhen you enter and exit sub screens.\nThis allows for releasing unused resources\nif sub-screens hold heavy references.",
                }
            }
        }
    }
}

struct SubScreen1 {}

impl Drop for SubScreen1 {
    fn drop(&mut self) {
        println!("Dropping SubScreen1");
    }
}

#[relm4::component]
//noinspection RsSortImplTraitMembers
impl SimpleComponent for SubScreen1 {
    type Input = ();
    type Output = ();
    type Init = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        println!("init SubScreen1");
        let model = Self {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_width_request: 500,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    set_label: "Sub screen 1",
                },
                gtk::Label {
                    set_label: "Imagine this screen is not used often,\nbut holds reference to something\nthat consumes a lot of memory.",
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: css::COMPONENT_SPACING,
                    gtk::Button {
                        set_label: "Back",
                        connect_clicked[sender] => move |_| sender.output(()).unwrap()
                    }
                }
            }
        }
    }
}

struct SubScreen2 {}

impl Drop for SubScreen2 {
    fn drop(&mut self) {
        println!("Dropping SubScreen2");
    }
}

#[relm4::component]
//noinspection RsSortImplTraitMembers
impl SimpleComponent for SubScreen2 {
    type Input = ();
    type Output = ();
    type Init = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        println!("init SubScreen2");
        let model = Self {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_height_request: 500,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    set_label: "Sub screen 2",
                },
                gtk::Label {
                    set_label: "Imagine this screen opens connection\nto remote host in order to show some live data.",
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: css::COMPONENT_SPACING,
                    gtk::Button {
                        set_label: "Back",
                        connect_clicked[sender] => move |_| sender.output(()).unwrap()
                    }
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.drop_sub_components");
    app.run::<App>(());
}
