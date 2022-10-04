use std::cell::RefCell;

use fragile::Fragile;
use once_cell::sync::Lazy;

type Callback = Box<dyn FnOnce()>;

static LATE_INIT: Lazy<Fragile<RefCell<Vec<Callback>>>> = Lazy::new(Fragile::default);

pub(super) fn register_callback(func: Callback) {
    LATE_INIT.get().borrow_mut().push(func);
}

pub(super) fn run_late_init() {
    LATE_INIT
        .get()
        .borrow_mut()
        .drain(..)
        .for_each(|func| func());
}
