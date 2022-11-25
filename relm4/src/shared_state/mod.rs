mod reducer;
mod state;

type SubscriberFn<Data> = Box<dyn Fn(&Data) -> bool + 'static + Send + Sync>;

pub use reducer::{Reducer, Reducible};
pub use state::{SharedState, SharedStateReadGuard, SharedStateWriteGuard};
