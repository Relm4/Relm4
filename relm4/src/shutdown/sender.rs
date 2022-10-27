// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use async_broadcast::Sender;

/// Sends shutdown signals to receivers.
#[derive(Debug)]
pub struct ShutdownSender {
    pub(super) sender: Sender<()>,
}

impl ShutdownSender {
    /// Broadcasts a shutdown signal to listening receivers.
    pub fn shutdown(&self) {
        drop(self.sender.try_broadcast(()));
    }
}

impl Drop for ShutdownSender {
    fn drop(&mut self) {
        self.shutdown();
    }
}
