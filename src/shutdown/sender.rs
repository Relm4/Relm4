// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MIT or Apache-2.0

use tokio::sync::broadcast::Sender;

/// Sends shutdown signals to receivers.
#[derive(Debug)]
pub struct ShutdownSender {
    pub(super) sender: Sender<()>,
}

impl ShutdownSender {
    /// Broadcasts a shutdown signal to listening receivers.
    pub fn shutdown(&self) {
        let _ = self.sender.send(());
    }
}

impl Clone for ShutdownSender {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
