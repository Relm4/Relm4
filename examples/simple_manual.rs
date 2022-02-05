use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt};
use relm4::{gtk, send, ComponentParts, RelmApp, Sender, SimpleComponent, WidgetPlus};

struct AppModel {
    counter: u8,
}

#[derive(Debug)]
enum AppMsg {
    Increment,
    Decrement,
}

struct AppWidgets {
    //window: gtk::Window,
    //vbox: gtk::Box,
    //inc_button: gtk::Button,
    //dec_button: gtk::Button,
    label: gtk::Label,
}

impl SimpleComponent for AppModel {
    type Widgets = AppWidgets;
    type Root = gtk::Window;

    type InitParams = u8;

    type Input = AppMsg;
    type Output = ();

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("Simple app")
            .default_width(300)
            .default_height(100)
            .build()
    }

    /// Initialize the UI.
    fn init_parts(
        counter: Self::InitParams,
        window: &Self::Root,
        input: &mut Sender<Self::Input>,
        _output: &mut Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets> {
        let model = AppModel { counter };

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .build();

        let inc_button = gtk::Button::with_label("Increment");
        let dec_button = gtk::Button::with_label("Decrement");

        let label = gtk::Label::new(Some(&format!("Counter: {}", model.counter)));
        label.set_margin_all(5);

        window.set_child(Some(&vbox));
        vbox.set_margin_all(5);
        vbox.append(&inc_button);
        vbox.append(&dec_button);
        vbox.append(&label);

        let btn_sender = input.clone();
        inc_button.connect_clicked(move |_| {
            send!(btn_sender, AppMsg::Increment);
        });

        let btn_sender = input.clone();
        dec_button.connect_clicked(move |_| {
            send!(btn_sender, AppMsg::Decrement);
        });

        let widgets = AppWidgets { label };

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _input: &mut Sender<Self::Input>,
        _ouput: &mut Sender<Self::Output>,
    ) {
        match msg {
            AppMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }

    /// Update the view to represent the updated model.
    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        _input: &mut Sender<Self::Input>,
        _output: &mut Sender<Self::Output>,
    ) {
        widgets
            .label
            .set_label(&format!("Counter: {}", self.counter));
    }
}

fn main() {
    let app: RelmApp<AppModel> = RelmApp::new("relm4.test.simple_manual");
    app.run(0);
}
