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
use relm4::{gtk, CommandFuture, Component, Fuselage, Sender};

#[tokio::main]
async fn main() {
    let app = gtk::builders::ApplicationBuilder::new()
        .application_id("org.relm4.SettingsListExample")
        .build();

    app.connect_activate(move |app| {
        let component = SettingsListModel::init()
            .launch("Settings List Demo".into())
            .connect_receiver(move |sender, message| match message {
                SettingsListOutput::Clicked(id) => {
                    eprintln!("ID {id} Clicked");

                    match id {
                        0 => xdg_open("https://github.com/AaronErhardt/Relm4".into()),
                        1 => xdg_open("https://aaronerhardt.github.io/docs/relm4/relm4/".into()),
                        2 => {
                            let _ = sender.send(SettingsListInput::Clear);
                        }
                        _ => (),
                    }
                }

                SettingsListOutput::Reload => {
                    let _ = sender.send(SettingsListInput::AddSetting {
                        description: "Browse GitHub Repository".into(),
                        button: "GitHub".into(),
                        id: 0,
                    });

                    let _ = sender.send(SettingsListInput::AddSetting {
                        description: "Browse Documentation".into(),
                        button: "Docs".into(),
                        id: 1,
                    });

                    let _ = sender.send(SettingsListInput::AddSetting {
                        description: "Clear List".into(),
                        button: "Clear".into(),
                        id: 2,
                    });
                }
            });

        relm4_macros::view! {
            window = gtk::ApplicationWindow {
                set_application: Some(app),
                set_child = Some(&gtk::Box) {
                    append: &component.widget,
                }
            }
        }

        window.show();
    });

    app.run();
}

#[derive(Default)]
pub struct SettingsListModel {
    pub options: Vec<(String, String, u32)>,
}

pub struct SettingsListWidgets {
    pub list: gtk::ListBox,
    pub options: Vec<gtk::Box>,
    pub button_sg: gtk::SizeGroup,
}

pub enum SettingsListInput {
    AddSetting {
        description: String,
        button: String,
        id: u32,
    },
    Clear,
    Reload,
}

pub enum SettingsListOutput {
    Clicked(u32),
    Reload,
}

pub enum SettingsListCommand {
    Reload,
}

impl Component for SettingsListModel {
    type Command = SettingsListCommand;
    type Input = SettingsListInput;
    type Output = SettingsListOutput;
    type Payload = String;
    type Root = gtk::Box;
    type Widgets = SettingsListWidgets;

    fn init_root() -> Self::Root {
        gtk::builders::BoxBuilder::new()
            .halign(gtk::Align::Center)
            .hexpand(true)
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn dock(
        title: String,
        root: &Self::Root,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) -> Fuselage<Self, Self::Widgets> {
        // Request the caller to reload its options.
        let _ = output.send(SettingsListOutput::Reload);

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

        Fuselage {
            model: SettingsListModel::default(),
            widgets: SettingsListWidgets {
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
            SettingsListInput::AddSetting {
                description,
                button,
                id,
            } => {
                self.options.push((description, button, id));
            }

            SettingsListInput::Clear => {
                self.options.clear();
                return Some(SettingsListCommand::Reload);
            }

            SettingsListInput::Reload => {
                let _ = output.send(SettingsListOutput::Reload);
            }
        }

        None
    }

    fn update_view(
        &self,
        widgets: &mut Self::Widgets,
        _input: &mut Sender<Self::Input>,
        output: &mut Sender<Self::Output>,
    ) {
        if self.options.is_empty() && !widgets.options.is_empty() {
            while let Some(child) = widgets.list.last_child() {
                widgets.list.remove(&child);
            }
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

                            connect_clicked(output) => move |_| {
                                let _ = output.send(SettingsListOutput::Clicked(id));
                            }
                        }
                    }
                }

                widgets.button_sg.add_widget(&button);
                widgets.list.append(&widget);
                widgets.options.push(widget);
            }
        }
    }

    fn command(message: Self::Command, input: Sender<Self::Input>) -> CommandFuture {
        Box::pin(async move {
            match message {
                SettingsListCommand::Reload => {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let _ = input.send(SettingsListInput::Reload);
                }
            }
        })
    }
}

fn xdg_open(item: String) {
    std::thread::spawn(move || {
        let _ = std::process::Command::new("xdg-open").arg(item).status();
    });
}
