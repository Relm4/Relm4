use gtk::cairo::{Context, Operator};
use gtk::prelude::{
    BoxExt, DrawingAreaExt, GestureSingleExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::{gtk, send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};

enum AppMsg {
    AddPoint((f64, f64)),
    UpdatePoints,
    Reset,
    Resize((i32, i32)),
}

struct AppModel {
    width: f64,
    height: f64,
    points: Vec<Point>,
    reset: bool,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = ();
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &(), _sender: Sender<AppMsg>) -> bool {
        self.reset = false;
        match msg {
            AppMsg::AddPoint((x, y)) => {
                self.points.push(Point::new(x, y));
            }
            AppMsg::UpdatePoints => {
                for point in &mut self.points {
                    let Point { x, y, .. } = point;
                    if *x < 0.0 {
                        point.xs = point.xs.abs();
                    } else if *x > self.width {
                        point.xs = -point.xs.abs();
                    }
                    *x = x.clamp(0.0, self.width);
                    *x += point.xs;

                    if *y < 0.0 {
                        point.ys = point.ys.abs();
                    } else if *y > self.height {
                        point.ys = -point.ys.abs();
                    }
                    *y = y.clamp(0.0, self.height);
                    *y += point.ys;
                }
            }
            AppMsg::Resize((x, y)) => {
                self.width = x as f64;
                self.height = y as f64;
            }
            AppMsg::Reset => {
                self.reset = true;
            }
        }
        true
    }
}

struct Point {
    x: f64,
    y: f64,
    xs: f64,
    ys: f64,
    color: Color,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        let angle: f64 = rand::random::<f64>() * std::f64::consts::PI * 2.0;
        Point {
            x,
            y,
            xs: angle.sin() * 7.0,
            ys: angle.cos() * 7.0,
            color: Color::random(),
        }
    }
}

struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    fn random() -> Color {
        Color {
            r: rand::random(),
            g: rand::random(),
            b: rand::random(),
        }
    }
}

fn draw(cx: &Context, points: &[Point]) {
    for point in points {
        let Point {
            x,
            y,
            color: Color { r, g, b },
            ..
        } = *point;
        cx.set_source_rgb(r, g, b);
        cx.arc(x, y, 10.0, 0.0, std::f64::consts::PI * 2.0);
        cx.fill().expect("Couldn't fill arc");
    }
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
      main_window = gtk::ApplicationWindow {
        set_default_height: 300,
        set_default_width: 600,
        set_child = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_all: 10,
          set_spacing: 10,
          set_hexpand: true,
          append = &gtk::Label {
            set_label: "Left-click to add circles, resize or right-click to reset!",
          },
          append: area = &gtk::DrawingArea {
            set_vexpand: true,
            set_hexpand: true,
            add_controller = &gtk::GestureClick::new() {
              set_button: 0,
              connect_pressed(sender) => move |controller, _, x, y| {
                if controller.current_button() == gtk::gdk::BUTTON_SECONDARY {
                  send!(sender, AppMsg::Reset);
                } else {
                  send!(sender, AppMsg::AddPoint((x, y)));
                }
              }
            },
            connect_resize(sender) => move |_, x, y| {
              send!(sender, AppMsg::Resize((x, y)))
            }
          },
        }
      }
    }

    additional_fields! {
        handler: relm4::drawing::DrawHandler
    }

    fn post_init() {
        let mut handler = relm4::drawing::DrawHandler::new().unwrap();
        handler.init(&area);

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(20));
            send!(sender, AppMsg::UpdatePoints);
        });
    }

    fn pre_view() {
        let cx = self.handler.get_context().unwrap();
        if model.reset {
            cx.set_operator(Operator::Clear);
            cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cx.paint().expect("Couldn't fill context");
        }
        draw(&cx, &model.points);
    }
}

fn main() {
    let model = AppModel {
        width: 100.0,
        height: 100.0,
        points: Vec::new(),
        reset: false,
    };
    let relm = RelmApp::new(model);
    relm.run();
}
