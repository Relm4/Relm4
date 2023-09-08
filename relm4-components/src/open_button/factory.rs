use gtk::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::{gtk, RelmWidgetExt};

use super::OpenButtonMsg;

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct FileListItem {
    pub(crate) path: PathBuf,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for FileListItem {
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

    fn init_model(init: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self { path: init }
    }
}
