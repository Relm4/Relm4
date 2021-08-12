use gtk::prelude::{
    BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::{send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

enum AppMsg {
    Increment,
}

#[tracker::track]
struct AppModel {
    width: u32,
    counter: u32,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        self.reset();
        match msg {
            AppMsg::Increment => {
                self.update_counter(|cnt| *cnt += 3);
            }
        }
        println!("counter: {}", self.counter);
        true
    }
}

#[relm4_macros::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
      main_window = gtk::ApplicationWindow {
        set_default_height: 400,
        set_default_width: model.width as i32,
        set_child = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_all: 10,
          set_spacing: 10,
          set_hexpand: true,
          append: area = &gtk::DrawingArea {
            set_vexpand: true,
            set_hexpand: true,
          },
          append: label = &gtk::Label {
            set_label: track!(model.changed(AppModel::counter()),
                &format!("Counter is at: {}", model.counter)),
          },
          append: button = &gtk::Button::new() {
            set_label: watch!{ &format!("Clicked: {}", model.counter)},
            set_visible: true,
            connect_clicked => move |_btn| {
              send!(sender, AppMsg::Increment);
            },
          },
        }
      }
    }

    additional_fields! { handler: relm4::drawing::DrawHandler }

    fn post_init() {
        let mut handler = relm4::drawing::DrawHandler::new().unwrap();
        handler.init(&area);
    }

    fn manual_view() {
        let cx = self.handler.get_context().unwrap();
        cx.set_source_rgb(model.counter as f64 / 100.0, 1.0, 0.0);
        cx.move_to(0.0, 0.0);
        cx.line_to(100.0, model.counter as f64);
        cx.stroke().unwrap();
    }
}

fn main() {
    let model = AppModel {
        width: 1000,
        counter: 0,
        tracker: 0,
    };
    let relm = RelmApp::new(model);
    relm.run();
}
