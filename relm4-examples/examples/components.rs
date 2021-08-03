use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use relm4::Sender;
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

impl_model!(Comp1Model, CompMsg);
impl_model!(Comp2Model, CompMsg);

#[derive(PartialEq)]
enum CompMsg {
    Hide,
    Show,
}

impl RelmWidgets for Comp1Widgets {
    type Root = gtk::Button;
    type Model = Comp1Model;

    fn init_view(model: &Comp1Model, _component: &(), sender: Sender<CompMsg>) -> Self {
        // Initialize gtk widgets
        let button = gtk::Button::with_label("First Component");
        button.set_visible(!model.hidden);

        button.connect_clicked(move |_button| {
            sender.send(CompMsg::Hide).unwrap();
        });

        Comp1Widgets { button }
    }

    fn view(&mut self, model: &Comp1Model, _sender: Sender<CompMsg>) {
        self.button.set_visible(!model.hidden);
    }

    fn root_widget(&self) -> Self::Root {
        self.button.clone()
    }
}

impl RelmWidgets for Comp2Widgets {
    type Root = gtk::Button;
    type Model = Comp2Model;

    fn init_view(model: &Comp2Model, _component: &(), sender: Sender<CompMsg>) -> Self {
        let button = gtk::Button::with_label("Second Component");
        button.set_visible(!model.hidden);

        button.connect_clicked(move |_button| {
            sender.send(CompMsg::Hide).unwrap();
        });

        Comp2Widgets { button }
    }

    fn view(&mut self, model: &Comp2Model, _sender: Sender<CompMsg>) {
        self.button.set_visible(!model.hidden);
    }

    fn root_widget(&self) -> Self::Root {
        self.button.clone()
    }
}

impl ComponentUpdate for Comp1Model {
    type ParentModel = AppModel;

    fn init_model(_parent_model: &AppModel) -> Self {
        Comp1Model { hidden: false }
    }

    fn update(
        &mut self,
        message: CompMsg,
        _components: &(),
        _sender: Sender<CompMsg>,
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
}

impl ComponentUpdate for Comp2Model {
    type ParentModel = AppModel;

    fn init_model(_parent_model: &AppModel) -> Self {
        Comp2Model { hidden: true }
    }

    fn update(
        &mut self,
        message: CompMsg,
        _components: &(),
        _sender: Sender<CompMsg>,
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
}

// The main app
struct Components {
    comp1: RelmComponent<Comp1Widgets>,
    comp2: RelmComponent<Comp2Widgets>,
}

impl RelmComponents<AppModel> for Components {
    fn init_components(parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        Components {
            comp1: RelmComponent::with_new_thread(parent_model, parent_sender.clone()),
            comp2: RelmComponent::new(parent_model, parent_sender),
        }
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::Label,
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

impl_model!(AppModel, AppMsg, Components);

impl RelmWidgets for AppWidgets {
    type Root = gtk::ApplicationWindow;
    type Model = AppModel;

    fn init_view(model: &AppModel, components: &Components, sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindowBuilder::new().build();
        let vbox = gtk::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();
        vbox.set_margin_all(5);

        let text = gtk::Label::new(Some(&model.counter.to_string()));

        let inc_button = gtk::Button::with_label("Increment");
        let dec_button = gtk::Button::with_label("Decrement");

        vbox.append(&text);
        vbox.append(&inc_button);
        vbox.append(&dec_button);

        vbox.append(components.comp1.root_widget());
        vbox.append(components.comp2.root_widget());

        main.set_child(Some(&vbox));

        let sender2 = sender.clone();

        inc_button.connect_clicked(move |_button| {
            sender.send(AppMsg::Increment).unwrap();
        });

        dec_button.connect_clicked(move |_button| {
            sender2.send(AppMsg::Decrement).unwrap();
        });

        AppWidgets { main, text }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        self.text.set_label(&model.counter.to_string());
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &Components, _sender: Sender<AppMsg>) {
        match msg {
            AppMsg::Increment => self.counter = self.counter.saturating_add(1),
            AppMsg::Decrement => self.counter = self.counter.saturating_sub(1),
            AppMsg::ShowComp1 => {
                components.comp1.send(CompMsg::Show).unwrap();
            }
            AppMsg::ShowComp2 => {
                components.comp2.send(CompMsg::Show).unwrap();
            }
        }
        println!("counter: {}", self.counter);
    }
}

fn main() {
    let model = AppModel { counter: 0 };
    let relm: RelmApp<AppWidgets> = RelmApp::new(model);
    relm.run();
}
