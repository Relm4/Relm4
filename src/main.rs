use glib::Sender;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use relm4::*;

// Implement components that will be part of the main app
struct Comp1Widgets {
    button: gtk::Button,
}

struct Comp2Widgets {
    button: gtk::Button,
}

struct Comp1Model {
    hidden: bool,
}

struct Comp2Model {
    hidden: bool,
}

#[derive(PartialEq)]
enum CompMsg {
    Hide,
    Show,
}

impl Widget<CompMsg, Comp1Model> for Comp1Widgets {
    type Root = gtk::Button;

    fn init_view(sender: Sender<CompMsg>, model: &Comp1Model) -> Self {
        // Initialize gtk widgets
        let button = gtk::Button::with_label("First Component");
        button.set_visible(!model.hidden);

        button.connect_clicked(move |_button| {
            sender.send(CompMsg::Hide).unwrap();
        });

        Comp1Widgets { button }
    }

    fn root_widget(&self) -> Self::Root {
        self.button.clone()
    }
}

impl Widget<CompMsg, Comp2Model> for Comp2Widgets {
    type Root = gtk::Button;

    fn init_view(sender: Sender<CompMsg>, model: &Comp2Model) -> Self {
        let button = gtk::Button::with_label("Second Component");
        button.set_visible(!model.hidden);

        button.connect_clicked(move |_button| {
            sender.send(CompMsg::Hide).unwrap();
        });

        Comp2Widgets { button }
    }

    fn root_widget(&self) -> Self::Root {
        self.button.clone()
    }
}

impl ComponentUpdate<CompMsg, AppMsg> for Comp1Model {
    type Widgets = Comp1Widgets;

    fn init_model() -> Self {
        Comp1Model { hidden: false }
    }

    fn update(
        &mut self,
        message: CompMsg,
        _widgets: &Self::Widgets,
        parent_sender: Sender<AppMsg>,
    ) {
        match message {
            CompMsg::Hide => {
                self.hidden = true;
                // Send message to parent
                parent_sender.send(AppMsg::ShowComp2).unwrap();
            }
            CompMsg::Show => {
                self.hidden = false;
            }
        }
    }

    fn view(&self, widgets: &mut Self::Widgets) {
        widgets.button.set_visible(!self.hidden);
    }
}

impl ComponentUpdate<CompMsg, AppMsg> for Comp2Model {
    type Widgets = Comp2Widgets;

    fn init_model() -> Self {
        Comp2Model { hidden: true }
    }

    fn update(
        &mut self,
        message: CompMsg,
        _widgets: &Self::Widgets,
        parent_sender: Sender<AppMsg>,
    ) {
        match message {
            CompMsg::Hide => {
                self.hidden = true;
                parent_sender.send(AppMsg::ShowComp1).unwrap();
            }
            CompMsg::Show => {
                self.hidden = false;
            }
        }
    }

    fn view(&self, widgets: &mut Self::Widgets) {
        widgets.button.set_visible(!self.hidden);
    }
}

// The main app
struct Components {
    comp1: RelmComponent<Comp1Widgets, Comp1Model, CompMsg, AppMsg>,
    comp2: RelmComponent<Comp2Widgets, Comp2Model, CompMsg, AppMsg>,
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::Label,
    relm: Components,
}

enum AppMsg {
    Increment,
    Decrement,
    ShowComp2,
    ShowComp1,
}

struct AppModel {
    counter: u8,
}

impl Widget<AppMsg, AppModel> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(sender: Sender<AppMsg>, model: &AppModel) -> Self {
        let main = gtk::ApplicationWindowBuilder::new().build();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .margin_end(5)
            .margin_top(5)
            .build();

        let text = gtk::Label::new(Some(&model.counter.to_string()));

        let inc_button = gtk::Button::with_label("Increment");
        let dec_button = gtk::Button::with_label("Decrement");

        let (comp1, comp1_root) = RelmComponent::create(sender.clone());
        let (comp2, comp2_root) = RelmComponent::create(sender.clone());

        vbox.append(&text);
        vbox.append(&inc_button);
        vbox.append(&dec_button);

        vbox.append(&comp1_root);
        vbox.append(&comp2_root);

        main.set_child(Some(&vbox));

        let sender2 = sender.clone();

        inc_button.connect_clicked(move |_button| {
            sender.send(AppMsg::Increment).unwrap();
        });

        dec_button.connect_clicked(move |_button| {
            sender2.send(AppMsg::Decrement).unwrap();
        });

        AppWidgets {
            main,
            text,
            relm: Components { comp1, comp2 },
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate<AppMsg> for AppModel {
    type Widgets = AppWidgets;

    fn init_model() -> Self {
        AppModel { counter: 0 }
    }

    fn update(&mut self, msg: AppMsg, widgets: &Self::Widgets) {
        match msg {
            AppMsg::Increment => self.counter = self.counter.saturating_add(1),
            AppMsg::Decrement => self.counter = self.counter.saturating_sub(1),
            AppMsg::ShowComp1 => {
                widgets.relm.comp1.sender().send(CompMsg::Show).unwrap();
            }
            AppMsg::ShowComp2 => {
                widgets.relm.comp2.sender().send(CompMsg::Show).unwrap();
            }
        }
        println!("counter: {}", self.counter);
    }

    fn view(&self, widgets: &mut Self::Widgets) {
        widgets.text.set_label(&self.counter.to_string());
    }
}

fn main() {
    gtk::init().unwrap();
    let relm: RelmApp<AppWidgets, AppModel, AppMsg> = RelmApp::create();
    relm.run();
}
