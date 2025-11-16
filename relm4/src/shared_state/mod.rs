//! Shared state that can be accessed by many components.

mod async_reducer;
mod reducer;
mod state;

type SubscriberFn<Data> = Box<dyn Fn(&Data) -> bool + 'static + Send + Sync>;

pub use async_reducer::{AsyncReducer, AsyncReducible};
pub use reducer::{Reducer, Reducible};
pub use state::{SharedState, SharedStateReadGuard, SharedStateWriteGuard};
