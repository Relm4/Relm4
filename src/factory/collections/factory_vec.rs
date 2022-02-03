use gtk::glib::Sender;

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::Widgets;
use crate::factory::{Factory, FactoryPrototype, FactoryView};

#[derive(Debug)]
enum ChangeType {
    Add,
    Remove,
    Recreate,
    Update,
}

/// A container similar to [`Vec`] that implements [`Factory`].
#[allow(clippy::type_complexity)]
#[derive(Default, Debug)]
pub struct FactoryVec<Data>
where
    Data: FactoryPrototype,
{
    data: Vec<Data>,
    widgets: RefCell<Vec<Widgets<Data::Widgets, <Data::View as FactoryView<Data::Root>>::Root>>>,
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
        while self.pop().is_some() {}
    }

    /// Returns the length as amount of elements stored in this type.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns [`true`] if the length of this type is `0`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // This comment explains the idea of how the state of this struct is kept
    // consistent. By modelling the state of each field in the vector as a state
    // machine, we have an easier time making sure that any operation performed
    // on the struct leaves it in a well-defined state. Here is a minimal
    // representation of this state-machine:
    //
    // `````````````````````````````````
    //     NoneO ⇄ Add
    //     NoneX → Remove ⇄ Recreate
    // `````````````````````````````````
    //
    // This state machine for a field in our struct consists of two kinds of
    // transitions (push, pop) and of these five states:
    // - [X ] Add        A widget is to be added for this field
    // - [O*] Remove     The widget for this field is to be removed
    // - [X*] Recreate   A new widget is replacing the current one
    // - [X*] NoneX      No change to occur, data and widget exist
    // - [O ] NoneO      No change to occur, no data nor widget
    //
    // Here the X annotation means, the field is currently backed by data, which
    // will be used to generate a widget on the next call to `generate`. On the
    // other hand, O means that it is *not* backed. The star * indicates there
    // currently exists a widget for this field. By assuming we can initially
    // start only in one of the two None states (i.e. with an empty change list)
    // and with the given reasonably well eyeballed transition types, no other
    // combinations can occur or are required. The two types of transitions
    // between states are adding and removing data, i.e., executing push or pop.
    // Adding can only start from states marked with O, while removing can only
    // happen from states marked with X. (The cases where these operations make
    // sense.)
    //
    // Note that the `Update` ChangeType is not explicitly included here. This
    // is because, regarding the transitions, it has the same semantics as NoneX
    // and can be treated equivalently.

    /// Insert an element at the end of a [`FactoryVec`].
    pub fn push(&mut self, data: Data) {
        let index = self.data.len();
        self.data.push(data);

        let change = match self.changes.borrow().get(&index) {
            None => ChangeType::Add,
            Some(ChangeType::Remove) => ChangeType::Recreate,
            Some(ChangeType::Add | ChangeType::Recreate | ChangeType::Update) => unreachable!(),
        };
        self.set_change(index, Some(change));
    }

    /// Remove an element at the end of a [`FactoryVec`].
    pub fn pop(&mut self) -> Option<Data> {
        let data = self.data.pop()?;
        let index = self.data.len();

        let change = match self.changes.borrow().get(&index) {
            None | Some(ChangeType::Update) => Some(ChangeType::Remove),
            Some(ChangeType::Add) => None,
            Some(ChangeType::Recreate) => Some(ChangeType::Remove),
            Some(ChangeType::Remove) => unreachable!(),
        };
        self.set_change(index, change);

        Some(data)
    }

    /// Get a reference to data stored at `index`.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Data> {
        self.data.get(index)
    }

    /// Get a mutable reference to data stored at `index`.
    ///
    /// Assumes that the data will be modified and the corresponding widget
    /// needs to be updated.
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Data> {
        let data = self.data.get_mut(index)?;
        let mut changes = self.changes.borrow_mut();
        changes.entry(index).or_insert(ChangeType::Update);
        Some(data)
    }

    /// Sets the change to be performed for a given index, None for reset.
    fn set_change(&self, index: usize, change: Option<ChangeType>) {
        if let Some(change) = change {
            self.changes.borrow_mut().insert(index, change);
        } else {
            self.changes.borrow_mut().remove(&index);
        }
    }
}

impl<Data, View> Factory<Data, View> for FactoryVec<Data>
where
    Data: FactoryPrototype<Factory = Self, View = View>,
    View: FactoryView<Data::Root>,
{
    type Key = usize;

    fn generate(&self, view: &View, sender: Sender<Data::Msg>) {
        // Compensate for removals changing the data under us.
        let mut neg_index_adjustment = 0;

        // Iterate from smallest to biggest index.
        for (index, change) in self.changes.borrow().iter() {
            let index = *index - neg_index_adjustment;
            let mut widgets = self.widgets.borrow_mut();

            match change {
                ChangeType::Add => {
                    let data = &self.data[index];
                    let new_widgets = data.init_view(&index, sender.clone());
                    let position = data.position(&index);
                    let root = view.add(Data::root_widget(&new_widgets), &position);
                    widgets.push(Widgets {
                        widgets: new_widgets,
                        root,
                    });
                }
                ChangeType::Update => {
                    self.data[index].view(&index, &widgets[index].widgets);
                }
                ChangeType::Remove => {
                    let remove_widget = widgets.remove(index);
                    view.remove(&remove_widget.root);
                    neg_index_adjustment += 1;
                }
                ChangeType::Recreate => {
                    let data = &self.data[index];
                    let new_widgets = data.init_view(&index, sender.clone());
                    let position = data.position(&index);
                    let root = view.add(Data::root_widget(&new_widgets), &position);
                    let remove_widget = std::mem::replace(
                        &mut widgets[index],
                        Widgets {
                            widgets: new_widgets,
                            root,
                        },
                    );
                    view.remove(&remove_widget.root);
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
