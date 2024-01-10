use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use gtk::gio::ApplicationFlags;
use gtk::glib::clone;
use gtk::prelude::{ApplicationExt, BoxExt, ButtonExt, GtkWindowExt};
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

// Iteration count that appear to be reasonable.
// Constant delays like GTK's runtime are negligible at this number.
const ITERATIONS: u32 = 200000;

struct AppModel {
    counter: u32,
    application: gtk::Application,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
}

struct AppWidgets {
    label: gtk::Label,
}

impl SimpleComponent for AppModel {
    type Init = gtk::Application;
    type Input = AppMsg;
    type Output = ();
    type Root = gtk::Window;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("Stress test benchmark")
            .default_width(300)
            .default_height(100)
            .build()
    }

    fn init(
        application: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            counter: 0,
            application,
        };

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .build();

        let stress_test_button = gtk::Button::with_label("Increment");

        let label = gtk::Label::new(Some(&format!("Counter: {}", model.counter)));
        label.set_margin_all(5);

        window.set_child(Some(&vbox));
        vbox.set_margin_all(5);
        vbox.append(&stress_test_button);
        vbox.append(&label);

        stress_test_button.connect_clicked(clone!(@strong sender => move |_| {
            for _ in 0..=ITERATIONS {
                sender.input(AppMsg::Increment);
            }
        }));

        let widgets = AppWidgets { label };

        stress_test_button.emit_clicked();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
        }
        if self.counter == ITERATIONS {
            self.application.quit();
        }
    }

    /// Update the view to represent the updated model.
    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets
            .label
            .set_label(&format!("Counter: {}", self.counter));
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(100))
        .sample_size(20);
    targets = benchmark
}
criterion_main!(benches);

fn benchmark(c: &mut Criterion) {
    c.bench_function("stress_test", move |b| {
        let application = gtk::Application::new(
            Some("relm4.bench.stress_test"),
            ApplicationFlags::FLAGS_NONE,
        );

        b.iter(move || {
            let app = RelmApp::from_app(application.clone());
            let application = application.clone();
            app.run::<AppModel>(application);
        })
    });
}
