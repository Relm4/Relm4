use adw::{prelude::AdwApplicationWindowExt, CenteringPolicy, ViewStackPage};
use gtk::{
    prelude::{
        BoxExt, ButtonExt, GtkWindowExt, ObjectExt, OrientableExt, ToggleButtonExt, WidgetExt,
    },
    Orientation,
};
use relm4::{adw, gtk, send, AppUpdate, Model, RelmApp, Sender, Widgets};

#[derive(Default)]
struct AppModel {
    counter: u8,
    attention: bool,
}

enum AppMsg {
    Increment,
    Decrement,
    Attention(bool),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
            AppMsg::Attention(v) => {
                self.attention = v;
            }
        }
        true
    }
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = adw::ApplicationWindow {
            set_default_width: 450,
            set_default_height: 200,
            set_content = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &adw::HeaderBar {
                    set_title_widget: title = Some(&adw::ViewSwitcherTitle) {
                        set_title: "A stack switcher App",
                        set_stack: Some(&stack),
                    },
                    set_centering_policy: CenteringPolicy::Strict,
                },
                append: stack = &adw::ViewStack {
                    set_vexpand: true,
                    add_titled(Some("First"), "First Page") = &gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_hexpand: false,
                        append = &gtk::Label {
                            set_label: "This is the start page",

                        },
                        append = &gtk::Button {
                            set_label: "Increase",
                            connect_clicked(sender) => move |_| {
                                send!(sender, AppMsg::Increment)
                            }
                        },
                        append = &gtk::Button {
                            set_label: "Decrease",
                            connect_clicked(sender) => move |_| {
                                send!(sender, AppMsg::Decrement)
                            }
                        },
                        append = &gtk::ToggleButton {
                            set_label: "Needs Attention",
                            set_active: model.attention,
                            connect_clicked(sender) => move |v| {
                                send!(sender, AppMsg::Attention(v.is_active()))
                            }
                        },
                    } -> first_page: ViewStackPage {
                        set_icon_name: Some("document-print-symbolic"),
                        set_needs_attention: watch!(model.attention),
                        set_badge_number: watch!(model.counter as u32),
                    },
                    add_titled(Some("Second"), "Second Page") = &gtk::Label {
                        set_label: "This is the second page"
                    } -> {
                        set_icon_name: Some("media-playback-start-symbolic"),
                        set_badge_number: 3,
                    },
                    add_titled(Some("Third"), "Third Page") = &gtk::Label {
                        set_label: "This is the last page"
                    } -> {
                        set_icon_name: Some("mypaint-brushes-symbolic")
                    },
                },
                append: bottom_bar = &adw::ViewSwitcherBar {
                    set_stack: Some(&stack),
                }
            },
        }
    }

    fn post_init() {
        title
            .bind_property("title-visible", &bottom_bar, "reveal")
            .flags(gtk::glib::BindingFlags::SYNC_CREATE)
            .build();
    }
}

fn main() {
    let model = AppModel {
        counter: 5,
        attention: true,
    };
    let app = RelmApp::new(model);
    app.run();
}
