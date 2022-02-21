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

use tokio::sync::broadcast;

/// Creates a broadcasting shutdown channel.
///
/// The sending side is responsible for initiating a shutdown.
/// The receiving side is responsible for responding to shutdowns.
pub fn channel() -> (ShutdownSender, ShutdownReceiver) {
    let alive = Arc::new(AtomicBool::new(true));
    let (sender, receiver) = broadcast::channel(1);
    (
        ShutdownSender {
            alive: alive.clone(),
            sender: sender.clone(),
        },
        ShutdownReceiver {
            alive,
            sender,
            receiver,
        },
    )
}
