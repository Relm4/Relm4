use std::time::Duration;

use gtk::cairo::{Context, Operator};
use gtk::prelude::*;
use relm4::drawing::DrawHandler;
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt};

#[derive(Debug)]
enum Msg {
    AddPoint((f64, f64)),
    Reset,
    Resize((i32, i32)),
}

#[derive(Debug)]
struct UpdatePointsMsg;

struct App {
    width: f64,
    height: f64,
    points: Vec<Point>,
    handler: DrawHandler,
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = Msg;
    type Output = ();
    type CommandOutput = UpdatePointsMsg;

    view! {
      gtk::Window {
        set_default_size: (600, 300),

        gtk::Box {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_all: 10,
          set_spacing: 10,
          set_hexpand: true,

          gtk::Label {
            set_label: "Left-click to add circles, resize or right-click to reset!",
          },

          #[local_ref]
          area -> gtk::DrawingArea {
            set_vexpand: true,
            set_hexpand: true,

            add_controller = gtk::GestureClick {
              set_button: 0,
              connect_pressed[sender] => move |controller, _, x, y| {
                if controller.current_button() == gtk::gdk::BUTTON_SECONDARY {
                    sender.input(Msg::Reset);
                } else {
                    sender.input(Msg::AddPoint((x, y)));
                }
              }
            },
            connect_resize[sender] => move |_, x, y| {
                sender.input(Msg::Resize((x, y)));
            }
          },
        }
      }
    }

    fn update(&mut self, msg: Msg, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let cx = self.handler.get_context();

        match msg {
            Msg::AddPoint((x, y)) => {
                self.points.push(Point::new(x, y));
            }
            Msg::Resize((x, y)) => {
                self.width = x as f64;
                self.height = y as f64;
            }
            Msg::Reset => {
                cx.set_operator(Operator::Clear);
                cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                cx.paint().expect("Couldn't fill context");
            }
        }

        draw(&cx, &self.points);
    }

    fn update_cmd(&mut self, _: UpdatePointsMsg, _: ComponentSender<Self>, _root: &Self::Root) {
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

        let cx = self.handler.get_context();
        draw(&cx, &self.points);
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            width: 100.0,
            height: 100.0,
            points: Vec::new(),
            handler: DrawHandler::new(),
        };

        let area = model.handler.drawing_area();
        let widgets = view_output!();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    loop {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        out.send(UpdatePointsMsg).unwrap();
                    }
                })
                .drop_on_shutdown()
        });

        ComponentParts { model, widgets }
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

fn main() {
    let app = RelmApp::new("relm4.examples.drawing");
    app.run::<App>(());
}
