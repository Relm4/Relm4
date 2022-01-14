use bytes::Buf;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, GtkWindowExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::{
    gtk, AppUpdate, AsyncComponentUpdate, AsyncRelmWorker, Model, RelmApp, Sender, Widgets,
};

struct HttpModel {}

impl Model for HttpModel {
    type Msg = HttpMsg;
    type Widgets = ();
    type Components = ();
}

enum HttpMsg {
    Request(String),
}

#[relm4::async_trait]
impl AsyncComponentUpdate<AppModel> for HttpModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        HttpModel {}
    }

    async fn update(
        &mut self,
        msg: HttpMsg,
        _components: &(),
        _sender: Sender<HttpMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            HttpMsg::Request(url) => {
                let t2_sender = parent_sender.clone();
                let favicon_url = format!(
                    "{}/favicon.ico",
                    url.splitn(4, '/').take(3).collect::<String>()
                );

                let t1 = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                    let image = if let Ok(req) = reqwest::get(favicon_url).await {
                        req.bytes().await.ok()
                    } else {
                        None
                    };
                    parent_sender.send(AppMsg::SetImage(image)).unwrap();
                });

                let t2 = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                    let text = if let Ok(req) = reqwest::get(url).await {
                        req.text().await.ok()
                    } else {
                        None
                    };
                    t2_sender.send(AppMsg::SetText(text)).unwrap();
                });

                let (_r1, _r2) = tokio::join!(t1, t2);
            }
        }
    }
}

#[derive(relm4::Components)]
struct AppComponents {
    http: AsyncRelmWorker<HttpModel, AppModel>,
}

#[tracker::track]
struct AppModel {
    text: String,
    text_waiting: bool,
    image_data: Option<Pixbuf>,
    image_waiting: bool,
}

#[derive(Debug)]
enum AppMsg {
    Request(String),
    SetText(Option<String>),
    SetImage(Option<bytes::Bytes>),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        self.reset();

        match msg {
            AppMsg::Request(url) => {
                components.http.send(HttpMsg::Request(url)).unwrap();
                self.set_text_waiting(true);
                self.set_image_waiting(true);
            }
            AppMsg::SetText(text_opt) => {
                self.set_text_waiting(false);
                if let Some(text) = text_opt {
                    self.set_text(text);
                } else {
                    self.set_text("No response".to_string());
                }
            }
            AppMsg::SetImage(bytes_opt) => {
                self.set_image_waiting(false);
                if let Some(bytes) = bytes_opt {
                    let buf = Pixbuf::from_read(bytes.reader()).ok();
                    self.set_image_data(buf);
                } else {
                    self.set_image_data(None);
                }
            }
        }
        true
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    image: gtk::Image,
    image_spinner: gtk::Spinner,
    text: gtk::TextView,
    text_window: gtk::ScrolledWindow,
    text_spinner: gtk::Spinner,
    text_spinner_box: gtk::CenterBox,
    submit: gtk::Button,
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(model: &AppModel, _components: &AppComponents, sender: Sender<AppMsg>) -> Self {
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

        let url = gtk::Entry::builder()
            .placeholder_text("https://example.com")
            .build();
        let submit = gtk::Button::with_label("Submit");

        let image = gtk::Image::builder()
            .height_request(40)
            .visible(!model.image_waiting)
            .build();
        let image_spinner = gtk::Spinner::builder()
            .spinning(true)
            .visible(model.image_waiting)
            .build();
        let image_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        image_box.append(&image);
        image_box.append(&image_spinner);

        let text = gtk::TextView::builder()
            .visible(!model.text_waiting)
            .build();
        let text_window = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        text_window.set_child(Some(&text));
        let text_spinner = gtk::Spinner::builder().spinning(true).build();
        let text_spinner_box = gtk::CenterBox::builder()
            .visible(model.text_waiting)
            .vexpand(true)
            .hexpand(true)
            .build();
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        text_spinner_box.set_center_widget(Some(&text_spinner));
        text_box.append(&text_window);
        text_box.append(&text_spinner_box);

        main_box.append(&url);
        main_box.append(&submit);
        main_box.append(&image_box);
        main_box.append(&text_box);

        main.set_child(Some(&main_box));

        submit.connect_clicked(move |_| {
            let text: String = url.text().into();
            sender.send(AppMsg::Request(text)).unwrap();
        });

        AppWidgets {
            main,
            image,
            image_spinner,
            text,
            text_window,
            text_spinner,
            text_spinner_box,
            submit,
        }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        if model.changed(AppModel::text()) {
            self.text.buffer().set_text(&model.text);
        }

        if model.changed(AppModel::text_waiting()) {
            self.text_window.set_visible(!model.text_waiting);
            self.text_spinner_box.set_visible(model.text_waiting);
            self.text_spinner.set_spinning(model.text_waiting);
            self.submit
                .set_sensitive(!model.image_waiting && !model.text_waiting);
        }

        if model.changed(AppModel::image_waiting()) {
            self.image.set_visible(!model.image_waiting);
            self.image_spinner.set_visible(model.image_waiting);
            self.image_spinner.set_spinning(model.image_waiting);
            self.submit
                .set_sensitive(!model.image_waiting && !model.text_waiting);
        }

        if model.changed(AppModel::image_data()) {
            if let Some(buf) = &model.image_data {
                self.image.set_from_pixbuf(Some(buf));
            } else {
                self.image.clear();
            }
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

fn main() {
    let model = AppModel {
        text: String::new(),
        text_waiting: false,
        image_data: None,
        image_waiting: false,
        tracker: 0,
    };
    let relm = RelmApp::new(model);
    relm.run();
}
