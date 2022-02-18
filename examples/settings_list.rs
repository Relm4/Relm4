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
    gtk::Application::builder()
        .application_id("org.relm4.SettingsListExample")
        .launch(|_app, window| {
            // Intiialize a component's root widget
            let component = App::init()
                // Attach the root widget to the given window.
                .attach_to(&window)
                // Start the component service with an initial parameter
                .launch("Settings List Demo".into())
                // Attach the returned receiver's messages to this closure.
                .connect_receiver(move |sender, message| match message {
                    Output::Clicked(id) => {
                        eprintln!("ID {id} Clicked");

                        match id {
                            0 => xdg_open("https://github.com/AaronErhardt/Relm4".into()),
                            1 => {
                                xdg_open("https://aaronerhardt.github.io/docs/relm4/relm4/".into())
                            }
                            2 => {
                                let _ = sender.send(Input::Clear);
                            }
                            _ => (),
                        }
                    }

                    Output::Reload => {
                        let _ = sender.send(Input::AddSetting {
                            description: "Browse GitHub Repository".into(),
                            button: "GitHub".into(),
                            id: 0,
                        });

                        let _ = sender.send(Input::AddSetting {
                            description: "Browse Documentation".into(),
                            button: "Docs".into(),
                            id: 1,
                        });

                        let _ = sender.send(Input::AddSetting {
                            description: "Clear List".into(),
                            button: "Clear".into(),
                            id: 2,
                        });
                    }
                });

            println!("parent is {:?}", component.widget().toplevel_window());
        });
}

#[derive(Default)]
pub struct App {
    pub options: Vec<(String, String, u32)>,
}

pub struct Widgets {
    pub list: gtk::ListBox,
    pub options: Vec<gtk::Box>,
    pub button_sg: gtk::SizeGroup,
}

pub enum Input {
    AddSetting {
        description: String,
        button: String,
        id: u32,
    },
    Clear,
    Reload,
}

pub enum Output {
    Clicked(u32),
    Reload,
}

pub enum Command {
    Reload,
}

pub enum CmdOut {
    Reload,
}

impl Component for App {
    type Command = Command;
    type CommandOutput = CmdOut;
    type Input = Input;
    type Output = Output;
    type InitParams = String;
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .halign(gtk::Align::Center)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn init_parts(
        title: Self::InitParams,
        root: &Self::Root,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> ComponentParts<Self, Self::Widgets> {
        // Request the caller to reload its options.
        let _ = output.send(Output::Reload);

        let label = gtk::builders::LabelBuilder::new()
            .label(&title)
            .margin_top(24)
            .build();

        let list = gtk::builders::ListBoxBuilder::new()
            .halign(gtk::Align::Center)
            .margin_bottom(24)
            .margin_top(24)
            .selection_mode(gtk::SelectionMode::None)
            .build();

        root.append(&label);
        root.append(&list);

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                list,
                button_sg: gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
                options: Default::default(),
            },
        }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Option<Self::Command> {
        match message {
            Input::AddSetting {
                description,
                button,
                id,
            } => {
                self.options.push((description, button, id));
            }

            Input::Clear => {
                self.options.clear();
                return Some(Command::Reload);
            }

            Input::Reload => {
                let _ = output.send(Output::Reload);
            }
        }

        None
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
        match message {
            CmdOut::Reload => {
                let _ = output.send(Output::Reload);
            }
        }
    }

    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
        if self.options.is_empty() && !widgets.options.is_empty() {
            widgets.list.remove_all();
        } else if self.options.len() != widgets.options.len() {
            if let Some((description, button_label, id)) = self.options.last() {
                let id = *id;
                relm4_macros::view! {
                    widget = gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_margin_start: 20,
                        set_margin_end: 20,
                        set_margin_top: 8,
                        set_margin_bottom: 8,
                        set_spacing: 24,

                        append = &gtk::Label {
                            set_label: description,
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_valign: gtk::Align::Center,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                        },

                        append: button = &gtk::Button {
                            set_label: button_label,
                            set_size_group: &widgets.button_sg,

                            connect_clicked(output) => move |_| {
                                let _ = output.send(Output::Clicked(id));
                            }
                        }
                    }
                }

                widgets.list.append(&widget);
                widgets.options.push(widget);
            }
        }
    }

    fn command(
        message: Self::Command,
        shutdown: ShutdownReceiver,
        out: Sender<CmdOut>,
    ) -> CommandFuture {
        shutdown
            // Performs this operation until a shutdown is triggered
            .register(async move {
                match message {
                    Command::Reload => {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        let _ = out.send(CmdOut::Reload);
                    }
                }
            })
            // Perform task until a shutdown interrupts it
            .wait_then_drop()
            // Wrap into a `Pin<Box<Future>>` for return
            .boxed()
    }
}

fn xdg_open(item: String) {
    std::thread::spawn(move || {
        let _ = std::process::Command::new("xdg-open").arg(item).status();
    });
}
