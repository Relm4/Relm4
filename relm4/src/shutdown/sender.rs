// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::broadcast;

/// Sends shutdown signals to receivers.
#[derive(Debug)]
pub struct ShutdownSender {
    pub(super) alive: Arc<AtomicBool>,
    pub(super) sender: broadcast::Sender<()>,
}

impl ShutdownSender {
    /// Broadcasts a shutdown signal to listening receivers.
    pub fn shutdown(&self) {
        drop(self.sender.send(()));
    }
}

impl Drop for ShutdownSender {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        self.shutdown();
    }
}
