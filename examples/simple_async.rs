use std::time::Duration;

use gtk::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    temp_widgets::TempWidgets,
    view, RelmApp, RelmWidgetExt,
};

struct App {
    counter: u8,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = u8;
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Button {
                    set_label: "Increment",
                    connect_clicked => Msg::Increment,
                },

                gtk::Button {
                    set_label: "Decrement",
                    connect_clicked => Msg::Decrement,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Counter: {}", model.counter),
                    set_margin_all: 5,
                }
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<TempWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some("Simple app"),
                set_default_size: (300, 100),

                // This will replaced by the Box of the fully
                // initialized view because Window can only have one child.
                // If the root of the component was a Box which can have
                // several children, you'd need to remove this again in init().
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(TempWidgets::new(root, spinner))
    }

    // Initialize the component.
    async fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        async_std::task::sleep(Duration::from_secs(1)).await;

        let model = App { counter };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        async_std::task::sleep(Duration::from_secs(1)).await;
        match msg {
            Msg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            Msg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.simple_async");
    app.run_async::<App>(0);
}
