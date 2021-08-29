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

/// A container similar to [`Vec`] that implements [`Factory`].
#[derive(Default, Debug)]
pub struct FactoryVec<Data>
where
    Data: FactoryPrototype,
{
    data: Vec<Data>,
    widgets: RefCell<Vec<Data::Widgets>>,
    changes: RefCell<BTreeMap<usize, ChangeType>>,
}

impl<Data> FactoryVec<Data>
where
    Data: FactoryPrototype,
{
    /// Create a new [`FactoryVec`].
    #[must_use]
    pub fn new() -> Self {
        FactoryVec {
            data: Vec::new(),
            widgets: RefCell::new(Vec::new()),
            changes: RefCell::new(BTreeMap::new()),
        }
    }

    /// Initialize a new [`FactoryVec`] with a normal [`Vec`].
    #[must_use]
    pub fn from_vec(data: Vec<Data>) -> Self {
        let changes = (0..data.len())
            .map(|data| (data, ChangeType::Add))
            .collect();
        let length = data.len();
        FactoryVec {
            data,
            widgets: RefCell::new(Vec::with_capacity(length)),
            changes: RefCell::new(changes),
        }
    }

    /// Get a slice of the internal data of a [`FactoryVec`].
    #[must_use]
    pub fn as_slice(&self) -> &[Data] {
        self.data.as_slice()
    }

    /// Get the internal data of the [`FactoryVec`].
    #[must_use]
    pub fn into_vec(self) -> Vec<Data> {
        self.data
    }

    /// Remove all data from the [`FactoryVec`].
    pub fn clear(&mut self) {
        for item in self.changes.borrow_mut().values_mut() {
            *item = ChangeType::Remove;
        }
        self.data.clear();
    }

    /// Returns the length as amount of elements stored in this type.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns [`true`] if the length of this type is `0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Insert an element at the end of a [`FactoryVec`].
    pub fn push(&mut self, data: Data) {
        let index = self.data.len();
        self.data.push(data);

        let change = match self.changes.borrow().get(&index) {
            Some(ChangeType::Recreate | ChangeType::Remove) => ChangeType::Recreate,
            _ => ChangeType::Add,
        };
        self.changes.borrow_mut().insert(index, change);
    }

    /// Remove an element at the end of a [`FactoryVec`].
    pub fn pop(&mut self) -> Option<Data> {
        let data = self.data.pop();
        if data.is_some() {
            let index = self.data.len();
            self.changes.borrow_mut().insert(index, ChangeType::Remove);
        }

        data
    }

    /// Get a reference to data stored at `index`.
    pub fn get(&self, index: usize) -> Option<&Data> {
        self.data.get(index)
    }

    /// Get a mutable reference to data stored at `index`.
    ///
    /// Assumes that the data will be modified and the corresponding widget
    /// needs to be updated.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Data> {
        let mut changes = self.changes.borrow_mut();
        changes.entry(index).or_insert(ChangeType::Update);

        self.data.get_mut(index)
    }
}

impl<Data, View> Factory<Data, View> for FactoryVec<Data>
where
    Data: FactoryPrototype<Factory = Self, View = View>,
    View: FactoryView<Data::Root>,
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
                    view.add(Data::get_root(&widget), &position);
                    widgets.push(widget);
                }
                ChangeType::Update => {
                    self.data[*index].update(index, &widgets[*index]);
                }
                ChangeType::Remove => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = Data::get_root(&widget);
                    view.remove(remove_widget);
                }
                ChangeType::Recreate => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = Data::get_root(&widget);
                    view.remove(remove_widget);
                    let data = &self.data[*index];
                    let widget = data.generate(index, sender.clone());
                    let position = data.position(index);
                    view.add(Data::get_root(&widget), &position);
                    widgets.push(widget);
                }
            }
        }
        self.changes.borrow_mut().clear();
    }
}

impl<Data, View> FactoryVec<Data>
where
    Data: FactoryPrototype<Factory = Self, View = View>,
    View: FactoryView<Data::Root>,
{
    /// Get an immutable iterator for this type
    pub fn iter(&self) -> std::slice::Iter<'_, Data> {
        self.data.iter()
    }
}
