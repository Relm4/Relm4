use gtk::glib::{object::Cast, Sender};
use gtk::prelude::{ButtonExt, GtkWindowExt};
use gtk::StringList;
use relm4::*;

struct AppWidgets {
    main: gtk::ApplicationWindow,
    cntr_box: gtk::CenterBox,
    toggle: gtk::ToggleButton,
}

enum AppMsg {
    Toggle,
    ShowLabel,
    ShowButton,
    ShowSlider,
}

enum WidgetSelection {
    Label,
    Button,
    Slider,
}

// The proc macro allows to easily track changes to different
// members of the model
#[tracker::track]
struct AppModel {
    counter: u8,
    #[no_eq]
    active_widget: WidgetSelection,
}

impl RelmWidgets<AppModel, (), AppMsg> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(model: &AppModel, _components: &(), sender: Sender<AppMsg>) -> Self {
        let main = gtk::ApplicationWindowBuilder::new()
            .default_width(300)
            .default_height(200)
            .build();
        let cntr_box = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_end(5)
            .margin_top(5)
            .margin_start(5)
            .margin_bottom(5)
            .build();

        let options = vec!["label", "button", "slider"];
        let list = StringList::new(&options);

        let drop_down = gtk::DropDown::builder().model(&list).build();
        let widget = gtk::Label::new(Some("Label"));
        let toggle = gtk::ToggleButton::with_label(&format!("Toggled: {}", model.counter));

        cntr_box.set_start_widget(Some(&drop_down));
        cntr_box.set_center_widget(Some(&widget));
        cntr_box.set_end_widget(Some(&toggle));
        main.set_child(Some(&cntr_box));

        let cloned_sender = sender.clone();
        drop_down.connect_selected_item_notify(move |drop_down| {
            let msg = match drop_down.selected() {
                0 => AppMsg::ShowLabel,
                1 => AppMsg::ShowButton,
                2 => AppMsg::ShowSlider,
                _ => return,
            };
            cloned_sender.send(msg).unwrap();
        });

        toggle.connect_clicked(move |_button| {
            sender.send(AppMsg::Toggle).unwrap();
        });

        AppWidgets {
            main,
            cntr_box,
            toggle,
        }
    }

    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        // Only update the widget if model.active_widget was actually changed.
        // This can be simply done by checking bits in the tracker variable of the member struct.
        if model.changed(AppModel::active_widget()) {
            let widget: gtk::Widget = match model.get_active_widget() {
                WidgetSelection::Label => gtk::Label::new(Some("Label")).upcast::<gtk::Widget>(),
                WidgetSelection::Button => {
                    gtk::Button::with_label("Button").upcast::<gtk::Widget>()
                }
                WidgetSelection::Slider => {
                    gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0)
                        .upcast::<gtk::Widget>()
                }
            };
            self.cntr_box.set_center_widget(Some(&widget));
        }
        // Only update toggle button if model.counter was actually changed
        if model.changed(AppModel::counter()) {
            self.toggle
                .set_label(&format!("Toggled: {}", model.counter));
        }
    }

    fn root_widget(&self) -> gtk::ApplicationWindow {
        self.main.clone()
    }
}

impl AppUpdate<(), AppMsg> for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) {
        // reset tracker value of the model
        self.reset();
        // set_#member_name() will set a bit in the tracker variable of the model
        match msg {
            AppMsg::Toggle => self.set_counter(self.get_counter().saturating_add(1)),
            AppMsg::ShowLabel => {
                self.set_active_widget(WidgetSelection::Label);
            }
            AppMsg::ShowButton => {
                self.set_active_widget(WidgetSelection::Button);
            }
            AppMsg::ShowSlider => {
                self.set_active_widget(WidgetSelection::Slider);
            }
        }
        println!("counter: {}", self.counter);
    }
}

fn main() {
    let model = AppModel {
        counter: 0,
        active_widget: WidgetSelection::Label,
        tracker: 0,
    };
    let relm: RelmApp<AppWidgets, AppModel, (), AppMsg> = RelmApp::new(model);
    relm.run();
}
