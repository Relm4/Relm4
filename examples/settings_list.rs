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

use gtk::prelude::*;
use relm4::*;

fn main() {
    gtk::Application::builder()
        .application_id("org.relm4.SettingsListExample")
        .launch(|_app, window| {
            // Initialize a component's root widget
            let mut component = App::builder()
                // Attach the root widget to the given window.
                .attach_to(&window)
                // Start the component service with an initial parameter
                .launch("Settings List Demo".into())
                // Attach the returned receiver's messages to this closure.
                .connect_receiver(move |sender, message| match message {
                    Output::Clicked(id) => {
                        eprintln!("ID {id} Clicked");

                        match id {
                            0 => xdg_open("https://github.com/Relm4/Relm4".into()),
                            1 => xdg_open("https://docs.rs/relm4/".into()),
                            2 => {
                                sender.send(Input::Clear).unwrap();
                            }
                            _ => (),
                        }
                    }

                    Output::Reload => {
                        sender
                            .send(Input::AddSetting {
                                description: "Browse GitHub Repository".into(),
                                button: "GitHub".into(),
                                id: 0,
                            })
                            .unwrap();

                        sender
                            .send(Input::AddSetting {
                                description: "Browse Documentation".into(),
                                button: "Docs".into(),
                                id: 1,
                            })
                            .unwrap();

                        sender
                            .send(Input::AddSetting {
                                description: "Clear List".into(),
                                button: "Clear".into(),
                                id: 2,
                            })
                            .unwrap();
                    }
                });

            // Keep runtime alive after the component is dropped
            component.detach_runtime();

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

#[derive(Debug)]
pub enum Input {
    AddSetting {
        description: String,
        button: String,
        id: u32,
    },
    Clear,
    Reload,
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
    Reload,
}

#[derive(Debug)]
pub enum CmdOut {
    Reload,
}

impl Component for App {
    type Init = String;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Widgets = Widgets;
    type Root = gtk::Box;

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .halign(gtk::Align::Center)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn init(
        title: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Request the caller to reload its options.
        sender.output(Output::Reload).unwrap();

        let label = gtk::Label::builder().label(title).margin_top(24).build();

        let list = gtk::ListBox::builder()
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

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
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

                // Perform this async operation.
                sender.oneshot_command(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    CmdOut::Reload
                });
            }

            Input::Reload => {
                sender.output(Output::Reload).unwrap();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CmdOut::Reload => {
                sender.output(Output::Reload).unwrap();
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        if self.options.is_empty() && !widgets.options.is_empty() {
            widgets.list.remove_all();
        } else if self.options.len() != widgets.options.len() {
            if let Some((description, button_label, id)) = self.options.last() {
                let id = *id;
                relm4::view! {
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

                            connect_clicked[sender] => move |_| {
                                sender.output(Output::Clicked(id)).unwrap();
                            }
                        }
                    }
                }

                widgets.list.append(&widget);
                widgets.options.push(widget);
            }
        }
    }
}

fn xdg_open(item: String) {
    std::thread::spawn(move || {
        let _ = std::process::Command::new("xdg-open").arg(item).status();
    });
}
