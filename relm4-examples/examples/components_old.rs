use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use relm4::{
    gtk, AppUpdate, ComponentUpdate, Components, Model, RelmApp, RelmComponent, Sender, WidgetPlus,
    Widgets,
};

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

impl Model for Comp1Model {
    type Msg = CompMsg;
    type Widgets = Comp1Widgets;
    type Components = ();
}

impl Model for Comp2Model {
    type Msg = CompMsg;
    type Widgets = Comp2Widgets;
    type Components = ();
}

#[derive(PartialEq)]
enum CompMsg {
    Hide,
    Show,
}

impl Widgets<Comp1Model, AppModel> for Comp1Widgets {
    type Root = gtk::Button;

    fn init_view(model: &Comp1Model, _components: &(), sender: Sender<CompMsg>) -> Self {
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

impl Widgets<Comp2Model, AppModel> for Comp2Widgets {
    type Root = gtk::Button;

    fn init_view(model: &Comp2Model, _components: &(), sender: Sender<CompMsg>) -> Self {
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

impl ComponentUpdate<AppModel> for Comp1Model {
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

impl ComponentUpdate<AppModel> for Comp2Model {
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

struct AppComponents {
    comp1: RelmComponent<Comp1Model, AppModel>,
    comp2: RelmComponent<Comp2Model, AppModel>,
}

impl Components<AppModel> for AppComponents {
    fn init_components(parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        AppComponents {
            comp1: RelmComponent::new(parent_model, parent_sender.clone()),
            comp2: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &AppWidgets) {
        self.comp1.connect_parent(parent_widgets);
        self.comp2.connect_parent(parent_widgets);
    }
}

struct AppWidgets {
    main: gtk::ApplicationWindow,
    text: gtk::Label,
    #[allow(dead_code)]
    vbox: gtk::Box,
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

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(model: &AppModel, components: &AppComponents, sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindow::default();
        let vbox = gtk::Box::builder()
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

        main.set_child(Some(&vbox));

        let sender2 = sender.clone();

        inc_button.connect_clicked(move |_button| {
            sender.send(AppMsg::Increment).unwrap();
        });

        dec_button.connect_clicked(move |_button| {
            sender2.send(AppMsg::Decrement).unwrap();
        });

        vbox.append(components.comp1.root_widget());
        vbox.append(components.comp2.root_widget());

        AppWidgets { main, text, vbox }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        self.text.set_label(&model.counter.to_string());
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
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
        true
    }
}

fn main() {
    let model = AppModel { counter: 0 };
    let relm = RelmApp::new(model);
    relm.run();
}
