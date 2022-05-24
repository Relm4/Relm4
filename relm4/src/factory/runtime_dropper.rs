use std::{cell::Cell, rc::Rc};

use gtk::glib;

/// A type that will drop a runtime behind a shared reference
/// when it is dropped.
pub(super) struct RuntimeDropper {
    rt: Rc<Cell<Option<glib::SourceId>>>,
}

impl RuntimeDropper {
    pub(super) fn new(rt: Rc<Cell<Option<glib::SourceId>>>) -> Self {
        Self { rt }
    }
}

impl Drop for RuntimeDropper {
    fn drop(&mut self) {
        if let Some(id) = self.rt.take() {
            id.remove();
        }
    }
}
