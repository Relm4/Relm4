use adw::prelude::*;
use relm4::{
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent,
};
use relm4_components::simple_adw_combo_row::SimpleComboRow;

#[derive(Debug)]
enum AppMsg {
    Selected(usize),
}

struct App {
    combo_row: Controller<SimpleComboRow<&'static str>>,
    selected_variant: usize,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        #[name = "app"]
        adw::Window {
            set_default_size: (300, 100),
            set_title: Some("Libadwaita SimpleComboRow"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_vexpand: true,

                #[name = "sidebar_header"]
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Libadwaita SimpleComboRow",
                    },
                    set_show_end_title_buttons: false,
                },

                adw::PreferencesGroup {
                    #[local_ref]
                    combo_row -> adw::ComboRow,
                },

                gtk::Text {
                    #[watch]
                    set_text: &format!("Variant {}", model.selected_variant + 1),
                }
            },
        }
    }

    fn update(&mut self, msg: Self::Input, _: ComponentSender<Self>) {
        match msg {
            AppMsg::Selected(selected) => self.selected_variant = selected,
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            combo_row: SimpleComboRow::builder()
                .launch(SimpleComboRow {
                    variants: vec!["Variant 1", "Variant 2"],
                    active_index: None,
                })
                .forward(sender.input_sender(), AppMsg::Selected),
            selected_variant: 0,
        };

        let combo_row = model.combo_row.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.adw_combo_box");
    app.run::<App>(());
}
