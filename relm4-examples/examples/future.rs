use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, GtkWindowExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::Sender;
use relm4::*;

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::TextView,
}

#[derive(Debug)]
enum AppMsg {
    Request(String),
    Response(String),
}

#[tracker::track]
struct AppModel {
    text: String,
    waiting: bool,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(_model: &AppModel, _components: &(), sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindowBuilder::new()
            .default_width(300)
            .default_height(200)
            .build();
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_end(5)
            .margin_top(5)
            .margin_start(5)
            .margin_bottom(5)
            .spacing(5)
            .build();

        let url = gtk::Entry::builder()
            .placeholder_text("https://example.com")
            .build();
        let submit = gtk::Button::with_label("Submit");

        let scroller = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        let text = gtk::TextView::new();
        scroller.set_child(Some(&text));

        main_box.append(&url);
        main_box.append(&submit);
        main_box.append(&scroller);

        main.set_child(Some(&main_box));

        submit.connect_clicked(move |_| {
            let text: String = url.text().into();
            sender.send(AppMsg::Request(text)).unwrap();
        });

        AppWidgets { main, text }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        if model.changed(AppModel::text()) {
            self.text.buffer().set_text(&model.text);
        }

        if model.changed(AppModel::waiting()) {
            self.main.set_sensitive(!model.waiting);
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), sender: Sender<AppMsg>) -> bool {
        self.reset();

        match msg {
            AppMsg::Request(url) => {
                self.set_waiting(true);

                let fut = async move {
                    let mut text = "Connection error".to_string();

                    if surf::Url::parse(&url).is_ok() {
                        if let Ok(mut req) = surf::get(url).await {
                            if let Ok(resp) = req.body_string().await {
                                text = resp;
                            }
                        }
                    }
                    sender.send(AppMsg::Response(text)).unwrap();
                };

                spawn_future(fut);
            }
            AppMsg::Response(text) => {
                self.set_text(text);
                self.set_waiting(false);
            }
        }

        true
    }
}

fn main() {
    let model = AppModel {
        text: String::new(),
        waiting: false,
        tracker: 0,
    };
    let relm = RelmApp::new(model);
    relm.run();
}
