use gtk::glib;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, RelmApp};
use std::path::PathBuf;
use std::sync::OnceLock;

// This example must be run with a path argument
// `cargo run --example open_files "/path/like/this.opus"`

static APP_ID: &str = "relm4.example.open_files";

pub static OPEN_FILE: OnceLock<PathBuf> = OnceLock::new();

fn main() {
    let gtk_app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    gtk_app.connect_open(|app, files, _hint| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                let _ = OPEN_FILE.set(path);
            }
        }
        app.activate();
    });

    // Here you can initialize icons or css
    // gtk_app.connect_startup(|_app| {
    //     initialize_custom_icons();
    // });

    let relm4_app = RelmApp::from_app(gtk_app);
    relm4_app.run::<AppModel>(());
}

#[tracker::track]
#[derive(Debug)]
struct AppModel {
    path: PathBuf,
}

#[derive(Debug)]
enum AppMsg {
    ChangePath(PathBuf),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_default_size: (200, 100),
            gtk::Box {
                gtk::Label {
                    #[watch]
                    set_label: &model.path.to_string_lossy()
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            tracker: 0,
            path: PathBuf::new(),
        };

        let widgets = view_output!();

        glib::idle_add_local_once({
            move || {
                if let Some(path) = crate::OPEN_FILE.get() {
                    sender.input(AppMsg::ChangePath(path.to_path_buf()));
                }
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            AppMsg::ChangePath(path) => self.set_path(path),
        }
    }
}
