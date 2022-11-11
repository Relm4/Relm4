use gtk::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryComponentSender};
use relm4::{gtk, RelmWidgetExt};

use super::OpenButtonMsg;

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct FileListItem {
    pub(crate) path: PathBuf,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for FileListItem {
    type ParentInput = OpenButtonMsg;
    type CommandOutput = ();
    type Input = ();
    type Init = PathBuf;
    type ParentWidget = gtk::Box;
    type Output = OpenButtonMsg;

    view! {
        gtk::ListBoxRow {
            gtk::Button {
                set_label: self.path.iter().last().expect("Empty path").to_str().unwrap(),
                set_margin_all: 0,
                connect_clicked[sender, index] => move |_| {
                    sender.output(OpenButtonMsg::OpenRecent(index.clone()));
                }
            }
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<Self::ParentInput> {
        Some(output)
    }

    fn init_model(init: Self::Init, _: &DynamicIndex, _: FactoryComponentSender<Self>) -> Self {
        Self { path: init }
    }
}
