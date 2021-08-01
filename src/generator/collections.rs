use gtk::glib::Sender;

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::generator::{Generator, GeneratorBlueprint, GeneratorWidget};

#[derive(Debug)]
enum ChangeType {
    Add,
    Remove,
    Recreate,
    Update,
}

pub struct VecGen<T, Widget, Positioning, Msg> {
    data: Vec<T>,
    widgets: RefCell<Vec<Widget>>,
    changes: RefCell<BTreeMap<usize, ChangeType>>,
    generator: GeneratorBlueprint<T, usize, Widget, Positioning, Msg>,
}

impl<T, Widget, Positioning, Msg> VecGen<T, Widget, Positioning, Msg> {
    pub fn new(generator: GeneratorBlueprint<T, usize, Widget, Positioning, Msg>) -> Self {
        VecGen {
            data: Vec::new(),
            widgets: RefCell::new(Vec::new()),
            changes: RefCell::new(BTreeMap::new()),
            generator,
        }
    }

    pub fn push(&mut self, data: T) {
        let index = self.data.len();
        self.data.push(data);

        let change = match self.changes.borrow().get(&index) {
            Some(ChangeType::Recreate | ChangeType::Remove) => ChangeType::Recreate,
            _ => ChangeType::Add,
        };
        self.changes.borrow_mut().insert(index, change);
    }

    pub fn pop(&mut self) -> Option<T> {
        let data = self.data.pop();
        if data.is_some() {
            let index = self.data.len();
            self.changes.borrow_mut().insert(index, ChangeType::Remove);
        }

        data
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        let mut changes = self.changes.borrow_mut();
        changes.entry(index).or_insert(ChangeType::Update);

        &mut self.data[index]
    }
}

impl<W, T, Widget, Positioning, Msg> Generator<W, T, Widget, Positioning, Msg>
    for VecGen<T, Widget, Positioning, Msg>
where
    W: GeneratorWidget<Widget, Positioning>,
{
    fn generate(&self, parent: &W, sender: Sender<Msg>) {
        for (index, change) in self.changes.borrow().iter().rev() {
            let mut widgets = self.widgets.borrow_mut();

            match change {
                ChangeType::Add => {
                    let (widget, position) =
                        (self.generator.generate)(&self.data[*index], index, sender.clone());
                    parent.add(&widget, &position);
                    widgets.push(widget);
                }
                ChangeType::Update => {
                    (self.generator.update)(&self.data[*index], index, &widgets[*index]);
                }
                ChangeType::Remove => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = (self.generator.remove)(&widget);
                    parent.remove(remove_widget);
                }
                ChangeType::Recreate => {
                    let widget = widgets.pop().unwrap();
                    let remove_widget = (self.generator.remove)(&widget);
                    parent.remove(remove_widget);
                    let (widget, position) =
                        (self.generator.generate)(&self.data[*index], index, sender.clone());
                    parent.add(&widget, &position);
                    widgets.push(widget)
                }
            }
        }
        self.changes.borrow_mut().clear();
    }
}
