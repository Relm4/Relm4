// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

mod attached;
mod receiver;
mod sender;

pub use self::attached::AttachedShutdown;
pub use self::receiver::ShutdownReceiver;
pub use self::sender::ShutdownSender;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Creates a broadcasting shutdown channel.
///
/// The sending side is responsible for initiating a shutdown.
/// The receiving side is responsible for responding to shutdowns.
#[must_use]
pub fn channel() -> (ShutdownSender, ShutdownReceiver) {
    let alive = Arc::new(AtomicBool::new(true));
    let (sender, receiver) = async_broadcast::broadcast(1);
    (
        ShutdownSender {
            alive: alive.clone(),
            sender,
        },
        ShutdownReceiver { alive, receiver },
    )
}
