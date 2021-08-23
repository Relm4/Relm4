use gtk::prelude::ButtonExt;
use relm4::factory::{DynamicIndex, FactoryPrototype, FactoryVecDeque};
use relm4::{send, WidgetPlus};

use super::OpenButtonMsg;

use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug)]
pub struct FileListItem {
    pub path: PathBuf,
}

pub struct FileListItemWidgets {
    label: gtk::Button,
    row: gtk::ListBoxRow,
}

impl FactoryPrototype for FileListItem {
    type Factory = FactoryVecDeque<Self>;
    type View = gtk::Box;
    type Widgets = FileListItemWidgets;
    type Root = gtk::ListBoxRow;
    type Msg = OpenButtonMsg;

    fn generate(&self, key: &Rc<DynamicIndex>, sender: relm4::Sender<Self::Msg>) -> Self::Widgets {
        let label = gtk::Button::with_label(
            self.path
                .iter()
                .last()
                .expect("Empty path")
                .to_str()
                .expect("Couldn't convert path to string"),
        );
        let row = gtk::ListBoxRow::builder().child(&label).build();

        label.inline_css(b"margin: 0;");

        let key = key.clone();
        label.connect_clicked(move |_| {
            send!(sender, OpenButtonMsg::OpenRecent(key.clone()));
        });
        FileListItemWidgets { label, row }
    }

    fn update(&self, _key: &Rc<DynamicIndex>, widgets: &Self::Widgets) {
        widgets.label.set_label(
            self.path
                .iter()
                .last()
                .expect("Empty path")
                .to_str()
                .expect("Couldn't convert path to string"),
        );
    }

    fn position(&self, _key: &Rc<DynamicIndex>) {}

    fn get_root(widgets: &Self::Widgets) -> &Self::Root {
        &widgets.row
    }
}
