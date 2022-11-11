use std::cell::RefCell;

use fragile::Fragile;
use once_cell::sync::Lazy;

type Callback = Box<dyn FnOnce()>;

static LATE_INIT: Lazy<Fragile<RefCell<Vec<Callback>>>> = Lazy::new(Fragile::default);

pub(super) fn register_callback(func: Callback) {
    if let Some(inner) = Lazy::get(&LATE_INIT) {
        // If `Lazy` was initialized and is not empty,
        // this means that `run_late_init` has run already.
        // In this case, we need to call the callback immediately,
        // otherwise it will never be executed.
        if inner.get().borrow().is_empty() {
            return func();
        }
    }
    LATE_INIT.get().borrow_mut().push(func);
}

pub(super) fn run_late_init() {
    LATE_INIT
        .get()
        .borrow_mut()
        .drain(..)
        .for_each(|func| func());
}
