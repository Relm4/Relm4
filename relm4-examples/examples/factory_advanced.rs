use gtk::glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt};
use relm4::factory::{DynamicIndex, Factory, FactoryPrototype, FactoryVecDeque};
use relm4::*;

use std::rc::Rc;

#[derive(Debug)]
enum AppMsg {
    AddFirst,
    RemoveLast,
    CountAt(Rc<DynamicIndex>),
    RemoveAt(Rc<DynamicIndex>),
    InsertBefore(Rc<DynamicIndex>),
    InsertAfter(Rc<DynamicIndex>),
}

struct Counter {
    counter: u8,
}

struct AppModel {
    data: FactoryVecDeque<Counter>,
    counter: u8,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::AddFirst => {
                self.data.push_front(Counter {
                    counter: self.counter,
                });
            }
            AppMsg::RemoveLast => {
                self.data.pop_back();
            }
            AppMsg::CountAt(index) => {
                if let Some(data) = self.data.get_mut(index.current_index()) {
                    data.counter = data.counter.wrapping_sub(1);
                }
            }
            AppMsg::RemoveAt(index) => {
                self.data.remove(index.current_index());
            }
            AppMsg::InsertBefore(index) => {
                self.data.insert(
                    index.current_index(),
                    Counter {
                        counter: self.counter,
                    },
                );
            }
            AppMsg::InsertAfter(index) => {
                self.data.insert(
                    index.current_index() + 1,
                    Counter {
                        counter: self.counter,
                    },
                );
            }
        }
        self.counter += 1;
        true
    }
}

struct FctryWidgets {
    hbox: gtk::Box,
    counter_button: gtk::Button,
}

impl FactoryPrototype for Counter {
    type Factory = FactoryVecDeque<Self>;
    type Widgets = FctryWidgets;
    type Root = gtk::Box;
    type View = gtk::Box;
    type Msg = AppMsg;

    fn generate(&self, index: &Rc<DynamicIndex>, sender: Sender<AppMsg>) -> FctryWidgets {
        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(5)
            .build();

        let counter_button = gtk::Button::with_label(&self.counter.to_string());
        let index: Rc<DynamicIndex> = index.clone();

        let remove_button = gtk::Button::with_label("Remove");
        let ins_above_button = gtk::Button::with_label("Add above");
        let ins_below_button = gtk::Button::with_label("Add below");

        hbox.append(&counter_button);
        hbox.append(&remove_button);
        hbox.append(&ins_above_button);
        hbox.append(&ins_below_button);

        {
            let sender = sender.clone();
            let index = index.clone();
            counter_button.connect_clicked(move |_| {
                send!(sender, AppMsg::CountAt(index.clone()));
            });
        }

        {
            let sender = sender.clone();
            let index = index.clone();
            remove_button.connect_clicked(move |_| {
                send!(sender, AppMsg::RemoveAt(index.clone()));
            });
        }

        {
            let sender = sender.clone();
            let index = index.clone();
            ins_above_button.connect_clicked(move |_| {
                send!(sender, AppMsg::InsertBefore(index.clone()));
            });
        }

        ins_below_button.connect_clicked(move |_| {
            send!(sender, AppMsg::InsertAfter(index.clone()));
        });

        FctryWidgets {
            hbox,
            counter_button,
        }
    }

    fn position(&self, _index: &Rc<DynamicIndex>) {}

    fn update(&self, _index: &Rc<DynamicIndex>, widgets: &FctryWidgets) {
        widgets.counter_button.set_label(&self.counter.to_string());
    }
    fn get_root(widget: &FctryWidgets) -> &gtk::Box {
        &widget.hbox
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    gen_box: gtk::Box,
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
            cloned_sender.send(AppMsg::AddFirst).unwrap();
        });

        remove.connect_clicked(move |_| {
            sender.send(AppMsg::RemoveLast).unwrap();
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

fn main() {
    let model = AppModel {
        data: FactoryVecDeque::new(),
        counter: 0,
    };

    let relm = RelmApp::new(model);
    relm.run();
}
