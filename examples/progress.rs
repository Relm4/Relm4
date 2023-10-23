// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

//! A component which allows the caller to define what options ae in its list.
//!
//! On init of the view, an output is sent to the caller to request to load widgets.
//!
//! Clicking a button will open the webpage to that option.
//!
//! Clicking the clear button will clear the list box and send a command to the
//! background that waits 2 seconds before issuing a reload command back to the
//! component, which forwards the reload command back to the caller of the
//! component, which then issues to reload the widgets again.

use futures::FutureExt;
use gtk::prelude::*;
use relm4::*;

fn main() {
    RelmApp::new("org.relm4.ProgressExample").run::<App>("Settings List Demo".into());
}

#[derive(Default)]
pub struct App {
    /// Tracks progress status
    computing: bool,

    /// Contains output of a completed task.
    task: Option<CmdOut>,
}

pub struct Widgets {
    button: gtk::Button,
    label: gtk::Label,
    progress: gtk::ProgressBar,
}

#[derive(Debug)]
pub enum Input {
    Compute,
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
}

#[derive(Debug)]
pub enum CmdOut {
    /// Progress update from a command.
    Progress(f32),
    /// The final output of the command.
    Finished(Result<String, ()>),
}

impl Component for App {
    type Init = String;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Widgets = Widgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        _args: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        relm4::view! {
            container = gtk::Box {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_width_request: 300,
                set_spacing: 12,
                set_margin_top: 4,
                set_margin_bottom: 4,
                set_margin_start: 12,
                set_margin_end: 12,
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_spacing: 4,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,

                    append: label = &gtk::Label {
                        set_xalign: 0.0,
                        set_label: "Find the answer to life:",
                    },

                    append: progress = &gtk::ProgressBar {
                        set_visible: false,
                    },
                },

                append: button = &gtk::Button {
                    set_label: "Compute",
                    connect_clicked => Input::Compute,
                }
            }
        }

        root.set_child(Some(&container));

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                label,
                button,
                progress,
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Compute => {
                self.computing = true;
                sender.command(|out, shutdown| {
                    shutdown
                        // Performs this operation until a shutdown is triggered
                        .register(async move {
                            let mut progress = 0.0;

                            while progress < 1.0 {
                                out.send(CmdOut::Progress(progress)).unwrap();
                                progress += 0.1;
                                tokio::time::sleep(std::time::Duration::from_millis(333)).await;
                            }

                            out.send(CmdOut::Finished(Ok("42".into()))).unwrap();
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
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
        if let CmdOut::Finished(_) = message {
            self.computing = false;
        }

        self.task = Some(message);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.button.set_sensitive(!self.computing);

        if let Some(ref progress) = self.task {
            match progress {
                CmdOut::Progress(p) => {
                    widgets.label.set_label("Searching for the answer...");
                    widgets.progress.set_visible(true);
                    widgets.progress.set_fraction(*p as f64);
                }
                CmdOut::Finished(result) => {
                    widgets.progress.set_visible(false);
                    widgets
                        .label
                        .set_label(&format!("The answer to life is: {result:?}"));
                }
            }
        }
    }
}
