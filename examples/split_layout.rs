use relm4::prelude::*;
use gtk::prelude::*;
use relm4::adw::prelude::*;
use tracker;

#[derive(Debug)]
enum AppMsg {
    ToggleSidebar,
    SetVisibleSidebar,
}

#[tracker::track]
struct AppModel {
    sidebar_width: i32,
    sidebar_visible:  bool,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 600,
            set_default_height: 400,

            #[wrap(Some)]
            set_content = &adw::OverlaySplitView
            {
                #[track(model.changed(AppModel::sidebar_visible()))]
                set_show_sidebar: model.sidebar_visible,
                #[wrap(Some)]
                set_sidebar = &gtk::Box {
                    #[track(model.changed(AppModel::sidebar_width()))]
                    set_width_request: model.sidebar_width,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 12,
                    append = &gtk::Button
                    {
                        set_label: "Resize of Sidebar",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::ToggleSidebar);
                        }
                    },
                },
                #[wrap(Some)]
                set_content = &adw::ToolbarView
                {
                    add_top_bar = &adw::HeaderBar
                    {
                        set_show_title: false,
                    },
                    #[wrap(Some)]
                    set_content = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        append =  &gtk::Button
                        {
                            set_label: "Set Visible of Sidebar",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppMsg::SetVisibleSidebar);
                            },
                            set_halign: gtk::Align::Center,
                            set_hexpand: false,
                            set_width_request: 150,
                        }
                    }
                },

            }
        }
    }
    fn update(&mut self, msg: AppMsg, _sender: ComponentSender<Self>) {
        // reset tracker value of the model
        self.reset();
        match msg
        {
            AppMsg::ToggleSidebar => {
                self.set_sidebar_width(if self.sidebar_width == 200 { 50 } else { 200 }); // if current width is 200 -> set to 50, otherwise set to 200
            }
            AppMsg::SetVisibleSidebar => {
                self.set_sidebar_visible(!self.sidebar_visible); // if visible -> hide, if hidden -> show
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let model = AppModel {
            sidebar_width: 200,
            sidebar_visible: true,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.split_layout");
    app.run::<AppModel>(());
}