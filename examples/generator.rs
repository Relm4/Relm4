use glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt};
use relm4::generator::{Generator, GeneratorBlueprint, VecGen};
use relm4::*;

struct AppWidgets {
    main: gtk::ApplicationWindow,
    gen_box: gtk::Box,
    sender: Sender<AppMsg>,
}

#[derive(Debug)]
enum AppMsg {
    Add,
    Remove,
    Clicked(usize),
}

struct AppModel {
    data: VecGen<u8, gtk::Button, (), AppMsg>,
    counter: u8,
}

impl Widget<AppMsg, AppModel> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(sender: Sender<AppMsg>, _model: &AppModel) -> Self {
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

        let gen_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_end(5)
            .margin_top(5)
            .margin_start(5)
            .margin_bottom(5)
            .spacing(5)
            .build();

        let add = gtk::Button::with_label("Add");
        let remove = gtk::Button::with_label("Remove");

        main_box.append(&add);
        main_box.append(&remove);
        main_box.append(&gen_box);

        main.set_child(Some(&main_box));

        let cloned_sender = sender.clone();
        add.connect_clicked(move |_| {
            cloned_sender.send(AppMsg::Add).unwrap();
        });

        let cloned_sender = sender.clone();
        remove.connect_clicked(move |_| {
            cloned_sender.send(AppMsg::Remove).unwrap();
        });

        AppWidgets {
            main,
            gen_box,
            sender,
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate<AppMsg> for AppModel {
    type Widgets = AppWidgets;

    fn init_model() -> Self {
        let generator = GeneratorBlueprint {
            generate: |data: &u8, index: &usize, sender| {
                let button = gtk::Button::with_label(&data.to_string());
                let index = *index;
                button.connect_clicked(move |_| {
                    sender.send(AppMsg::Clicked(index)).unwrap();
                });
                (button, ())
            },
            update: |data, _index, widget| {
                widget.set_label(&data.to_string());
            },
            remove: |widget| widget,
        };
        AppModel {
            data: VecGen::new(generator),
            counter: 0,
        }
    }

    fn update(&mut self, msg: AppMsg, _widgets: &Self::Widgets) {
        match msg {
            AppMsg::Add => {
                self.data.push(self.counter);
                self.counter += 1;
            }
            AppMsg::Remove => {
                self.data.pop();
            }
            AppMsg::Clicked(index) => {
                let data = self.data.get_mut(index);
                *data = data.wrapping_sub(1);
            }
        }
    }

    fn view(&self, widgets: &mut Self::Widgets) {
        self.data.generate(&widgets.gen_box, widgets.sender.clone());
    }
}

fn main() {
    gtk::init().unwrap();
    let relm: RelmApp<AppWidgets, AppModel, AppMsg> = RelmApp::create();
    relm.run();
}
