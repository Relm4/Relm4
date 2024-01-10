// Don't show GTK 4.10 deprecations.
// We can't replace them without raising the GTK requirement to 4.10.
#![allow(deprecated)]

use gtk::prelude::*;
use relm4::{prelude::*, Sender};

struct Dialog {
    buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
enum DialogMsg {
    Accept,
    Cancel,
}

#[relm4::component]
impl SimpleComponent for Dialog {
    type Init = ();
    type Input = DialogMsg;
    type Output = String;
    type Widgets = DialogWidgets;

    view! {
        #[root]
        dialog = gtk::MessageDialog {
            set_margin_all: 12,
            set_modal: true,
            set_text: Some("Enter a search query"),
            add_button: ("Search", gtk::ResponseType::Accept),
            present: (),

            connect_response[sender] => move |dialog, resp| {
                dialog.set_visible(false);
                sender.input(if resp == gtk::ResponseType::Accept {
                    DialogMsg::Accept
                } else {
                    DialogMsg::Cancel
                });
            }
        },
        dialog.content_area() -> gtk::Box {
            gtk::Entry {
                set_buffer: &model.buffer,
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dialog {
            buffer: gtk::EntryBuffer::default(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            DialogMsg::Accept => {
                sender.output(self.buffer.text().into()).unwrap();
            }
            DialogMsg::Cancel => {
                sender.output(String::default()).unwrap();
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: Sender<Self::Output>) {
        println!("Dialog shutdown");
    }
}

#[derive(Debug)]
enum AppMsg {
    StartSearch,
}

struct App {
    result: Option<String>,
    searching: bool,
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;
    type CommandOutput = Option<String>;

    view! {
        main_window = gtk::ApplicationWindow {
            set_default_size: (300, 100),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 12,

                if let Some(result) = &model.result {
                    gtk::LinkButton {
                        set_label: "Your search result",
                        #[watch]
                        set_uri: result,
                    }
                } else {
                    gtk::Label {
                        set_label: "Click the button to start a web-search"
                    }
                },
                gtk::Button {
                    set_label: "Start search",
                    connect_clicked => AppMsg::StartSearch,
                    #[watch]
                    set_sensitive: !model.searching,
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            result: None,
            searching: false,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            AppMsg::StartSearch => {
                self.searching = true;

                let stream = Dialog::builder()
                    .transient_for(root)
                    .launch(())
                    .into_stream();
                sender.oneshot_command(async move {
                    // Use the component as stream
                    let result = stream.recv_one().await;

                    if let Some(search) = result {
                        let response =
                            reqwest::get(format!("https://duckduckgo.com/lite/?q={search}"))
                                .await
                                .unwrap();
                        let response_text = response.text().await.unwrap();

                        // Extract the url of the first search result.
                        if let Some(url) = response_text.split("<a rel=\"nofollow\" href=\"").nth(1)
                        {
                            let url = url.split('\"').next().unwrap().replace("amp;", "");
                            Some(format!("https:{url}"))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.searching = false;
        self.result = message;
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.message_stream");
    app.run::<App>(());
}
