use gtk::prelude::ButtonExt;
use relm4::factory::{DynamicIndex, FactoryPrototype, FactoryVecDeque};
use relm4::{gtk, send, WidgetPlus};

use super::OpenButtonMsg;

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct FileListItem {
    pub(crate) path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct FileListItemWidgets {
    label: gtk::Button,
    row: gtk::ListBoxRow,
}

impl FactoryPrototype for FileListItem {
    type Factory = FactoryVecDeque<Self>;
    type View = gtk::Box;
    type Widgets = FileListItemWidgets;
    type Root = gtk::ListBoxRow;
    type Msg = OpenButtonMsg;

    fn init_view(&self, key: &DynamicIndex, sender: relm4::Sender<Self::Msg>) -> Self::Widgets {
        let label = gtk::Button::with_label(
            self.path
                .iter()
                .last()
                .expect("Empty path")
                .to_str()
                .expect("Couldn't convert path to string"),
        );
        let row = gtk::ListBoxRow::builder().child(&label).build();

        label.inline_css(b"margin: 0");

        let key = key.clone();
        label.connect_clicked(move |_| {
            send!(sender, OpenButtonMsg::OpenRecent(key.clone()));
        });
        FileListItemWidgets { label, row }
    }

    fn view(&self, _key: &DynamicIndex, widgets: &Self::Widgets) {
        widgets.label.set_label(
            self.path
                .iter()
                .last()
                .expect("Empty path")
                .to_str()
                .expect("Couldn't convert path to string"),
        );
    }

    fn position(&self, _key: &DynamicIndex) {}

    fn root_widget(widgets: &Self::Widgets) -> &Self::Root {
        &widgets.row
    }
}
