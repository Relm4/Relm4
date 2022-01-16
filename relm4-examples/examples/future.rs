use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryExt, GtkWindowExt, TextBufferExt, TextViewExt,
};
use relm4::{gtk, spawn_future, AppUpdate, Model, RelmApp, Sender, Widgets};

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

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), sender: Sender<AppMsg>) -> bool {
        self.reset();

        match msg {
            AppMsg::Request(entry) if !self.waiting => {
                self.set_waiting(true);
                let client = surf::client().with(surf::middleware::Redirect::new(10));

                let fut = async move {
                    let text = if surf::Url::parse(&entry).is_ok() {
                        if let Ok(mut req) = client.send(surf::get(entry)).await {
                            if let Ok(resp) = req.body_string().await {
                                resp
                            } else {
                                "Couldn't get response body".to_string()
                            }
                        } else {
                            "Couldn't send request".to_string()
                        }
                    } else {
                        "Couldn't parse entry".to_string()
                    };
                    sender.send(AppMsg::Response(text)).unwrap();
                };

                spawn_future(fut);
            }
            AppMsg::Response(text) => {
                self.set_text(text);
                self.set_waiting(false);
            }
            _ => { /* Do nothing while waiting for a response */ }
        }

        true
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::TextView,
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(_model: &AppModel, _components: &(), sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindow::builder()
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

        let entry = gtk::Entry::builder()
            .placeholder_text("https://example.com")
            .build();
        let submit = gtk::Button::with_label("Submit");

        let scroller = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        let text = gtk::TextView::new();
        scroller.set_child(Some(&text));
        text.set_editable(false);

        main_box.append(&entry);
        main_box.append(&submit);
        main_box.append(&scroller);

        main.set_child(Some(&main_box));

        {
            let sender = sender.clone();
            let entry = entry.clone();
            submit.connect_clicked(move |_| {
                let text: String = entry.text().into();
                sender.send(AppMsg::Request(text)).unwrap();
            });
        }

        entry.connect_activate(move |entry| {
            let text: String = entry.text().into();
            sender.send(AppMsg::Request(text)).unwrap();
        });

        AppWidgets { main, text }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        if model.changed(AppModel::text()) {
            self.text.buffer().set_text(&model.text);
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
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
