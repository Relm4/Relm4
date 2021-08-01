use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::send;
use relm4::{AppUpdate, RelmApp, RelmWidgets, Sender};

enum AppMsg {
    Increment,
}

#[tracker::track]
struct AppModel {
    width: u32,
    counter: u32,
}

impl AppUpdate<(), AppMsg> for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) {
        self.reset();
        // reset tracker value of the model
        //self.reset();
        // set_#member_name() will set a bit in the tracker variable of the model
        match msg {
            AppMsg::Increment => {
                self.update_counter(|cnt| *cnt += 1);
            }
        }
        //println!("counter: {}", self.counter);
    }
}

#[relm4_macros::widget]
impl RelmWidgets for AppWidgets {
    // specify generic types
    type Model = AppModel;
    type Components = ();
    type Msg = AppMsg;

    view! {
      main_window = gtk::ApplicationWindow {
        set_default_height: 400,
        set_default_width: model.width as i32,
        set_child = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_start: 10,
          set_margin_end: 10,
          set_margin_top: 10,
          set_margin_bottom: 10,
          set_spacing: 10,
          set_hexpand: true,
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
          }
        }
      }
    }
}

fn main() {
    let model = AppModel {
        width: 1000,
        counter: 0,
        tracker: 0,
    };
    let relm: RelmApp<AppWidgets, AppModel, (), AppMsg> = RelmApp::new(model);
    relm.run();
}
