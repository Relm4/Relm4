use gtk::glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt};
use relm4::generator::{Generator, GeneratorBlueprint, VecGen};
use relm4::*;

struct AppWidgets {
    main: gtk::ApplicationWindow,
    gen_box: gtk::Box,
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

impl RelmWidgets<AppModel, (), AppMsg> for AppWidgets {
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

        remove.connect_clicked(move |_| {
            sender.send(AppMsg::Remove).unwrap();
        });

        AppWidgets { main, gen_box }
    }

    fn view(&mut self, model: &AppModel, sender: Sender<AppMsg>) {
        model.data.generate(&self.gen_box, sender);
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate<(), AppMsg> for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) {
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
}

fn main() {
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
    let model = AppModel {
        data: VecGen::new(generator),
        counter: 0,
    };

    let relm: RelmApp<AppWidgets, AppModel, (), AppMsg> = RelmApp::new(model);
    relm.run();
}
