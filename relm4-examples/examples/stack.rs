use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

#[derive(Clone, Copy)]
enum Page {
    Hello,
    Intro,
    Overview,
}

impl Page {
    fn next(self) -> Self {
        match self {
            Page::Hello => Page::Intro,
            Page::Intro => Page::Overview,
            Page::Overview => Page::Overview,
        }
    }

    fn previous(self) -> Self {
        match self {
            Page::Hello => Page::Hello,
            Page::Intro => Page::Hello,
            Page::Overview => Page::Intro,
        }
    }
}

struct AppModel {
    page: Page,
}

enum AppMsg {
    NextPage,
    PreviousPage,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::NextPage => {
                self.page = self.page.next();
            }
            AppMsg::PreviousPage => {
                self.page = self.page.previous();
            }
        }
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        gtk::ApplicationWindow {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                append: stack = &gtk::Stack {
                    set_margin_all: 5,
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,

                    add_child: hello_label = &gtk::Label {
                        set_label: "Hello! Welcome to this application!",
                    },
                    add_child: intro_label = &gtk::Label {
                        set_label: "To get started click on \"Next page\".",
                    },
                    add_child: overview_label = &gtk::Label {
                        set_label: "* Insert overview here *",
                    },
                },

                append = &gtk::Button {
                    set_label: "Next page",
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::NextPage);
                    },
                },
                append = &gtk::Button::with_label("Previous page") {
                    connect_clicked(sender) => move |_| {
                        send!(sender, AppMsg::PreviousPage);
                    },
                },
            },
        }
    }

    fn manual_view() {
        match model.page {
            Page::Hello => self.stack.set_visible_child(&self.hello_label),
            Page::Intro => self.stack.set_visible_child(&self.intro_label),
            Page::Overview => self.stack.set_visible_child(&self.overview_label),
        }
    }
}

fn main() {
    let model = AppModel { page: Page::Hello };
    let app = RelmApp::new(model);
    app.run();
}
