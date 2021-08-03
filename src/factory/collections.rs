use gtk::glib::Sender;

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::factory::{Factory, FactoryPrototype, FactoryView};

#[derive(Debug)]
enum ChangeType {
    Add,
    Remove,
    Recreate,
    Update,
}

#[derive(Default)]
pub struct FactoryVec<Data>
where
    Data: FactoryPrototype,
{
    data: Vec<Data>,
    widgets: RefCell<Vec<Data::Widget>>,
    changes: RefCell<BTreeMap<usize, ChangeType>>,
}

impl<Data> FactoryVec<Data>
where
    Data: FactoryPrototype,
{
    pub fn new() -> Self {
        FactoryVec {
            data: Vec::new(),
            widgets: RefCell::new(Vec::new()),
            changes: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn push(&mut self, data: Data) {
        let index = self.data.len();
        self.data.push(data);

        let change = match self.changes.borrow().get(&index) {
            Some(ChangeType::Recreate | ChangeType::Remove) => ChangeType::Recreate,
            _ => ChangeType::Add,
        };
        self.changes.borrow_mut().insert(index, change);
    }

    pub fn pop(&mut self) -> Option<Data> {
        let data = self.data.pop();
        if data.is_some() {
            let index = self.data.len();
            self.changes.borrow_mut().insert(index, ChangeType::Remove);
        }

        data
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Data {
        let mut changes = self.changes.borrow_mut();
        changes.entry(index).or_insert(ChangeType::Update);

        &mut self.data[index]
    }
}

impl<Data, View> Factory<Data, View> for FactoryVec<Data>
where
    Data: FactoryPrototype<Factory = Self, View = View>,
    View: FactoryView<Data::Widget>,
{
    type Key = usize;

    fn generate(&self, view: &View, sender: Sender<Data::Msg>) {
        for (index, change) in self.changes.borrow().iter().rev() {
            let mut widgets = self.widgets.borrow_mut();

            match change {
                ChangeType::Add => {
                    let data = &self.data[*index];
                    let widget = data.generate(index, sender.clone());
                    let position = data.position(index);
                    view.add(&widget, &position);
                    widgets.push(widget);
                }
                ChangeType::Update => {
                    self.data[*index].update(index, &widgets[*index]);
                }
                ChangeType::Remove => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = Data::remove(&widget);
                    view.remove(remove_widget);
                }
                ChangeType::Recreate => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = Data::remove(&widget);
                    view.remove(remove_widget);
                    let data = &self.data[*index];
                    let widget = data.generate(index, sender.clone());
                    let position = data.position(index);
                    view.add(&widget, &position);
                    widgets.push(widget);
                }
            }
        }
        self.changes.borrow_mut().clear();
    }
}
